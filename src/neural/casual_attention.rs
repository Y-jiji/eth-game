use tch::*;

#[derive(Debug)]
pub struct CausalAttention {
    c_attn: nn::Linear,
    c_proj: nn::Linear,
    n_head: i64,
    n_embd: i64,
    device: Device,
}

// copied from tch-rs repository
impl CausalAttention {
    pub fn new(vs: nn::Path, n_head: i64, n_embd: i64) -> Self {
        let c = nn::LinearConfig { bias: false, ..Default::default() };
        let c_attn = nn::linear(&vs / "c_attn", n_embd, 3 * n_embd, c);
        let c_proj = nn::linear(&vs / "c_proj", n_embd, n_embd, c);
        Self { c_attn, c_proj, n_head, n_embd, device: vs.device() }
    }

    pub fn apply_rotary_emb(&self, x: &Tensor, freqs_cis: &Tensor) -> Tensor {
        let mut dims = x.size();
        let v = dims.pop().unwrap();
        dims.push(v / 2);
        dims.push(2);
        let x = x.reshape(&dims);
        let re_x = x.slice(-1, 0, 1, 1);
        let im_x = x.slice(-1, 1, 2, 1);
        let re_f = freqs_cis.slice(-1, 0, 1, 1);
        let im_f = freqs_cis.slice(-1, 1, 2, 1);
        let re = &re_x * &re_f - &im_x * &im_f;
        let im = &re_x * &im_f + &im_x * &re_f;
        let rope = Tensor::cat(&[&re, &im], -1);
        // TODO: Add the flatten op.
        let mut dims = rope.size();
        let v1 = dims.pop().unwrap();
        let v2 = dims.pop().unwrap();
        dims.push(v1 * v2);
        rope.reshape(&dims)
    }

    pub fn forward(&self, x: &Tensor, freqs_cis: &Tensor) -> Tensor {
        use tch::nn::Module;
        let (b, t, c) = x.size3().unwrap();
        let kind = x.kind();
        let qkv = self.c_attn.forward(x);
        let n_embd = self.n_embd;
        let q = qkv.slice(2, 0, n_embd, 1);
        let k = qkv.slice(2, n_embd, 2 * n_embd, 1);
        let v = qkv.slice(2, 2 * n_embd, 3 * n_embd, 1);
        let target_dim = [b, t, self.n_head, c / self.n_head];
        let k = k.reshape(target_dim).transpose(1, 2);
        let q = q.reshape(target_dim).transpose(1, 2);
        let v = v.reshape(target_dim).transpose(1, 2);
        let q = self.apply_rotary_emb(&q, freqs_cis);
        let k = self.apply_rotary_emb(&k, freqs_cis);
        let k_shape = k.size();
        let att: Tensor = q.matmul(&k.transpose(-2, -1)) / (*k_shape.last().unwrap() as f64).sqrt();
        let mask = Tensor::ones([t, t], (kind, self.device)).tril(0).reshape([1, 1, t, t]);
        let att = att.masked_fill(&mask.eq(0.), f64::NEG_INFINITY);
        let y = att.softmax(-1, kind).matmul(&v);
        let y = y.transpose(1, 2).reshape([b, t, c]);
        self.c_proj.forward(&y)
    }
}