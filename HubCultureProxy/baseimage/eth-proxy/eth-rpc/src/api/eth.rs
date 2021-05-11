use types::{Filter,Log,Bytes,BlockId,Block,Transaction,TxInfo,TxCall,Receipt,U256,H256};
use api::{Request,Response,Expect,AsyncRpc};
use crypto::Address;
use rpc;

/// The `eth_*` api namespace.
///
#[derive(Debug,Clone)]
pub struct Eth<'a,T: 'a> {
    transport: &'a T
}


impl<'a,T> Eth<'a,T> {

    pub fn new(transport: &'a T) -> Self { Self { transport } }
}


impl<'a,T> Eth<'a,T> where T: rpc::Transport<Request,Response> {

    /// Equivalent to the `eth_getBlockByNumber` method.
    ///
    pub fn get_block_by_number(&self, block: BlockId) -> AsyncRpc<T::Future,Option<Block<H256>>> {
        let req = Request::get_block_by_number(block,false);
        self.execute(req)
    }

    /// Equivalent to the `eth_getTransactionByHash` method.
    ///
    pub fn get_tx_by_hash(&self, hash: H256) -> AsyncRpc<T::Future,Option<TxInfo>> {
        let req = Request::get_tx_by_hash(hash);
        self.execute(req)
    }

    /// Equivalent to the `eth_getTransactionReceipt` method.
    ///
    pub fn get_tx_receipt(&self, hash: H256) -> AsyncRpc<T::Future,Option<Receipt>> {
        let req = Request::get_tx_receipt(hash);
        self.execute(req)
    }

    /// Equivalent to the `eth_getLogs` method.
    ///
    pub fn get_logs(&self, filter: Filter) -> AsyncRpc<T::Future,Vec<Log>> {
        let req = Request::get_logs(filter);
        self.execute(req)
    }

    /// Equivalent to the `eth_getBalance` method.
    ///
    pub fn get_balance(&self, addr: Address, block: BlockId) -> AsyncRpc<T::Future,U256> {
        let req = Request::get_balance(addr,block);
        self.execute(req)
    }

    /// Equivalent to the `eth_getTransactionCount` method.
    ///
    pub fn get_tx_count(&self, addr: Address, block: BlockId) -> AsyncRpc<T::Future,U256> {
        let req = Request::get_tx_count(addr,block);
        self.execute(req)
    }

    /// Equivalent to the `eth_estimateGas` method.
    ///
    pub fn estimate_gas(&self, tx: Transaction, block: BlockId) -> AsyncRpc<T::Future,U256> {
        let req = Request::estimate_gas(tx,block);
        self.execute(req)
    }

    /// Equivalent to the `eth_call` method.
    ///
    pub fn call(&self, tx: TxCall, block: BlockId) -> AsyncRpc<T::Future,Bytes> {
        let req = Request::call(tx,block);
        self.execute(req)
    }

    /// Equivalent to the `eth_sendRawTransaction` method.
    ///
    pub fn send_raw_tx(&self, bytes: Bytes) -> AsyncRpc<T::Future,H256> {
        let req = Request::send_raw_tx(bytes);
        self.execute(req)
    }

    /// Equivalent to the `eth_blockNumber` method.
    ///
    pub fn block_number(&self) -> AsyncRpc<T::Future,U256> {
        let req = Request::block_number();
        self.execute(req)
    }

    /// Equivalent to the `eth_gasPrice` method.
    ///
    pub fn gas_price(&self) -> AsyncRpc<T::Future,U256> {
        let req = Request::gas_price();
        self.execute(req)
    }

    /// Equivalent to `eth_accounts` method.
    ///
    pub fn accounts(&self) -> AsyncRpc<T::Future,Vec<Address>> {
        let req = Request::accounts();
        self.execute(req)
    }

    fn execute<I>(&self, request: Request) -> AsyncRpc<T::Future,I> where Response: Expect<I> {
        AsyncRpc::new(self.transport.call(request))
    }
}
