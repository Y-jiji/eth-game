pub mod interfaces;
pub mod inspector;
use revm::db::*;
use revm::primitives::*;
use self::interfaces::Attacker;
use self::interfaces::Defender;
use once_cell::sync::Lazy;

pub struct Environment<A: Attacker, D: Defender, const TRACE: bool=false> {
    db: CacheDB<EmptyDB>,
    limit: usize,
    contracts: Vec<(B160, Bytes)>,
    attacker: (Option<B160>, Option<A>),
    defender: Option<D>,
}

static INIT_CODE: Lazy<Bytes> = Lazy::new(|| {
    const CODE: &str = "6080604052603e80600f5f395ff3fe60806040525f80fdfea2646970667358221220f4c053055368b5ab057d67cae6f8601779dfd4609d49738731a6c86d70e1f85464736f6c63430008150033";
    Bytes::from(hex::decode(CODE).unwrap())
});

impl<A: Attacker, D: Defender, const TRACE: bool> Environment<A, D, TRACE> {
    pub fn new(limit: usize) -> Self {
        Self { db: CacheDB::new(EmptyDB::default()), attacker: (None, None), defender: None, contracts: Vec::new(), limit }
    }
    pub fn get_contracts(&self) -> &[(B160, Bytes)] {
        &self.contracts
    }
    pub fn create_attacker_account(&mut self) -> B160 {
        // create an attacker account
        let mut evm = revm::EVM::new();
        let admin = B160::from(rand::random::<u64>() as u64);
        self.db.insert_account_info(
            admin, 
            AccountInfo {
                balance: U256::from(u64::MAX), 
                nonce: 0, 
                code_hash: KECCAK_EMPTY, 
                code: None 
            }
        );
        evm.database(&mut self.db);
        evm.env.tx.caller = admin;
        evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);
        evm.env.tx.data = Bytes::from(INIT_CODE.as_ref());
        evm.env.tx.value = U256::from(u64::MAX);
        let result = evm.transact_commit().unwrap();
        let addr = match result {
            ExecutionResult::Success { output: Output::Create(_, Some(address)), .. } 
                => address,
            result => panic!("contract creation failed: {result:?}"),
        };
        self.attacker.0 = Some(addr);
        return addr;
    }
    // load attacker
    pub fn load_attacker(&mut self, attacker: A) {
        self.attacker.1 = Some(attacker);
    }
    // load defender
    pub fn load_defender(&mut self, defender: D) {
        self.defender = Some(defender);
    }
    // load real world account information
    pub fn load_accounts(&mut self, targets: Vec<(B160, AccountInfo)>) {
        // insert account information into database
        for (address, info) in targets {
            self.contracts.push((address, info.code.as_ref().unwrap().bytes().clone()));
            self.db.insert_account_info(address, info);
        }
    }
    // load contract bytecode
    pub fn load_contracts(&mut self, target_init_codes: Vec<Bytes>) {
        // create an administrator account
        let admin = B160::from(rand::random::<u64>());
        // add an administator account
        self.db.insert_account_info(
            admin, 
            AccountInfo {
                balance: U256::from(u64::MAX), 
                nonce: 0,
                code_hash: KECCAK_EMPTY, 
                code: None
            }
        );
        // length of target init codes
        let len = target_init_codes.len() as u64;
        // load contract initialization code
        for init_code in target_init_codes {
            let mut evm = revm::EVM::new();
            evm.database(&mut self.db);
            evm.env.tx.caller = admin;
            evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);
            evm.env.tx.data = init_code;
            evm.env.tx.value = U256::from(0);
            evm.env.tx.gas_limit = u64::MAX;
            let result = evm.transact_commit().unwrap();
            let (code, address) = match result {
                ExecutionResult::Success { output: Output::Create(code, Some(address)), .. } => (code, address),
                result => panic!("contract creation failed: {result:?}"),
            };
            self.contracts.push((address, code));
            let mut evm = revm::EVM::new();
            evm.database(&mut self.db);
            evm.env.tx.caller = admin;
            evm.env.tx.transact_to = TransactTo::Call(address);
            evm.env.tx.data = Bytes::default();
            evm.env.tx.value = U256::from(u64::MAX / len);
            evm.env.tx.gas_limit = u64::MAX;
            let result = evm.transact_commit().unwrap();
            match result {
                ExecutionResult::Success { .. } => (),
                result => panic!("cannot transfer money to target contract {result:?}"),
            }
        }
    }
    // compute attacker initial value
    pub fn attacker_balance(&mut self) -> U256 {
        self.db.load_account(self.attacker.0.unwrap()).unwrap().info.balance.clone()
    }
    // compute attacker final value
    pub fn compute(mut self) -> U256 {
        // create an administrator account
        let admin = B160::from(rand::random::<u64>());
        // add an administator account
        self.db.insert_account_info(
            admin, 
            AccountInfo {
                balance: U256::from(u64::MAX), 
                nonce: 0, 
                code_hash: KECCAK_EMPTY, 
                code: None 
            }
        );
        // initialize attacker and defender with contracts
        let attstate = self.attacker.1.as_mut().unwrap().init(&self.contracts);
        let defstate = vec![self.defender.as_mut().unwrap().init(&self.contracts)];
        let inspector = inspector::GameInspector::<D, A, TRACE> {
            limit: self.limit,
            accounts: (HashSet::from_iter(self.contracts.iter().map(|x| x.0)), self.attacker.0.unwrap()),
            attacker: self.attacker.1.unwrap(), 
            defender: self.defender.unwrap(), 
            attstate, defstate
        };
        // call attacker to start the game, addr is attacker address
        let mut evm = revm::EVM::new();
        evm.database(&mut self.db);
        evm.env.tx.caller = admin;
        evm.env.tx.transact_to = TransactTo::Call(self.attacker.0.unwrap());
        evm.env.tx.data = Bytes::default();
        evm.env.tx.value = U256::from(0);
        evm.inspect_commit(inspector).unwrap();
        // give the final utility
        self.db.load_account(self.attacker.0.unwrap()).unwrap().info.balance.clone()
    }
}