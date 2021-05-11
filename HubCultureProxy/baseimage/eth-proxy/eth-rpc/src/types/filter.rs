use types::helpers::ValOrSeq;
use types::{BlockId,H256};
use smallvec::SmallVec;
use crypto::Address;


#[derive(Default,Debug,Clone)]
pub struct FilterBuilder {
    inner: Option<Filter>
}


impl FilterBuilder {

    fn filter_mut(&mut self) -> &mut Filter {
        self.inner.get_or_insert_with(Default::default)
    }

    pub fn from_block(&mut self, block: BlockId) -> &mut Self {
        self.filter_mut().from_block = Some(block);
        self
    }

    pub fn to_block(&mut self, block: BlockId) -> &mut Self {
        self.filter_mut().to_block = Some(block);
        self
    }

    pub fn topics(&mut self, a: Option<Topic>, b: Option<Topic>, c: Option<Topic>, d: Option<Topic>) -> &mut Self {
        let topics = match (&a,&b,&c,&d) {
            (_,_,_,Some(_)) => smallvec![a,b,c,d],
            (_,_,Some(_),None) => smallvec![a,b,c],
            (_,Some(_),None,None) => smallvec![a,b],
            (Some(_),None,None,None) => smallvec![a],
            (None,None,None,None) => smallvec![],
        };
        self.filter_mut().topics = Some(topics);
        self
    }

    pub fn origin(&mut self, origin: impl Into<Origin>) -> &mut Self {
        self.filter_mut().address = Some(origin.into());
        self
    }

    pub fn blockhash(&mut self, hash: H256) -> &mut Self {
        self.filter_mut().blockhash = Some(hash);
        self
    }

    pub fn finish(&mut self) -> Filter {
        self.inner.take().unwrap_or_default()
    }
}

#[derive(Default,Debug,Clone,Serialize,Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub from_block: Option<BlockId>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub to_block: Option<BlockId>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub topics: Option<Topics>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub address: Option<Origin>,
    #[serde(default,skip_serializing_if = "Option::is_none")]
    pub blockhash: Option<H256>,
}


impl Filter {

    pub fn builder() -> FilterBuilder { Default::default() }
}

pub type Origin = ValOrSeq<Address>;

pub type Topics = SmallVec<[Option<Topic>;4]>;

pub type Topic = ValOrSeq<H256>;

