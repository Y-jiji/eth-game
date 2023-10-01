pub mod interfaces;
pub mod inspector;
use revm::db::*;
use revm::primitives::*;
use self::interfaces::Attacker;
use self::interfaces::Defender;
use once_cell::sync::Lazy;

pub struct Environment<A: Attacker, D: Defender> {
    db: CacheDB<EmptyDB>,
    contracts: Vec<(B160, Bytes)>,
    attacker: A,
    defender: D,
}

static INIT_CODE: Lazy<Bytes> = Lazy::new(|| {
    Bytes::from(hex::decode(
        String::new() +
        "6080604052348015600f57600080fd5b" +
        "50603f80601d6000396000f3fe608060" +
        "4052600080fdfea26469706673582212" +
        "203e9cd2e4b65a21d520d56991fdd1bc" +
        "3ef05a91ad81ef47da4ff48e58372c44" +
        "0164736f6c63430008120033"
    ).unwrap())
});

impl<A: Attacker, D: Defender> Environment<A, D> {
    pub fn new(attacker: A, defender: D) -> Self {
        Self { db: CacheDB::new(EmptyDB::default()), attacker, defender, contracts: Vec::new() }
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
                balance: U256::MAX / U256::from(2), 
                nonce: 0, 
                code_hash: KECCAK_EMPTY, 
                code: None 
            }
        );
        // length of target init codes
        let len = target_init_codes.len();
        // load contract initialization code
        for init_code in target_init_codes {
            let mut evm = revm::EVM::new();
            evm.database(&mut self.db);
            evm.env.tx.caller = admin;
            evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);
            evm.env.tx.data = init_code;
            evm.env.tx.value = U256::MAX / U256::from(2 * len);
            let result = evm.transact_commit().unwrap();
            let (code, address) = match result {
                ExecutionResult::Success { output: Output::Create(code, Some(address)), .. } => (code, address),
                result => panic!("contract creation failed: {result:?}"),
            };
            self.contracts.push((address, code));
        }
    }
    // compute utility
    pub fn compute(mut self) -> U256 {
        // create an attacker account
        let mut evm = revm::EVM::new();
        let admin = B160::from(rand::random::<u64>());
        self.db.insert_account_info(
            admin, 
            AccountInfo {
                balance: U256::MAX / U256::from(2), 
                nonce: 0, 
                code_hash: KECCAK_EMPTY, 
                code: None 
            }
        );
        evm.database(&mut self.db);
        evm.env.tx.caller = admin;
        evm.env.tx.transact_to = TransactTo::Create(CreateScheme::Create);
        evm.env.tx.data = Bytes::from(INIT_CODE.as_ref());
        evm.env.tx.value = U256::MAX / U256::from(2);
        let result = evm.transact_commit().unwrap();
        let addr = match result {
            ExecutionResult::Success { output: Output::Create(_, Some(address)), .. } 
                => address,
            result => panic!("contract creation failed: {result:?}"),
        };
        // initialize attacker and defender with contracts
        self.attacker.init(&self.contracts);
        self.defender.init(&self.contracts);
        // call attacker to start the game, addr is attacker address
        let mut evm = revm::EVM::new();
        evm.database(&mut self.db);
        evm.env.tx.caller = admin;
        evm.env.tx.transact_to = TransactTo::Call(addr);
        evm.env.tx.data = Bytes::default();
        evm.env.tx.value = U256::from(0);
        evm.transact_commit().unwrap();
        // give the final utility
        self.db.load_account(addr).unwrap().info.balance.clone()
    }
}