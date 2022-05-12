use std::convert::TryInto;

use crate::msg::{HandleMsg, InitMsg, MessageResponse, QueryMsg, ViewingPermissions, HandleAnswer};
use crate::state::{config, append_message, create_empty_collection, Message, State, /*PERFIX_PERMITS*/ PREFIX_MSGS_RECEIVED, CONFIG_KEY, load, write_viewing_key};
use crate::viewing_key::ViewingKey;
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdError, StdResult, Storage, ReadonlyStorage,
};

use cosmwasm_storage::{ReadonlyPrefixedStorage, PrefixedStorage};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};

// HandleMsg::InitAddress
pub fn try_init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let ha = deps.api.human_address(&deps.api.canonical_address(&env.message.sender)?)?;
    //let adr = String::from(ha.clone().as_str());
    let message1 = Message::new(String::from("Hasbulla/homefolder/pepe.jpg"), String::from("Bi.jpg"));
    //creating an AppendStore collection for sender with a dummy file for testing purposes
    append_message(&mut deps.storage, &message1, &ha);

    //creating an empty Appendstore collection for sender 
    //create_empty_collection(& mut deps.storage, &ha);
    
    // let mut store = PrefixedStorage::multilevel(&[PREFIX_MSGS, ha.0.as_bytes()], &mut deps.storage);
    // let store = AppendStore::<Message, _, _>::attach(&store);

    //Register Wallet info - may not need to do this
    // let wallet_info = WalletInfo { 
    //     init : true
    // };
    // let bucket_response = bucket(FILE_LOCATION, &mut deps.storage).save(&adr.as_bytes(), &wallet_info);
    // match bucket_response {
    //     Ok(bucket_response) => bucket_response,
    //     Err(e) => panic!("Bucket Error: {}", e)
    // }

    // Let's create viewing key - creates a viewing key for whoever made this collection
    let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;
    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());
    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    write_viewing_key(&mut deps.storage, &message_sender, &key);
    
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

pub fn try_create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

//if you have queried to retrieve a vector of all the files, you know what position your desired file is at 
//so you can throw in the position and get the file - is this enough for front end to make buttons to retrieve a single message which can access a file?
//consider writing fn to get file given a contents: String instead of position? Would be a lot harder 

pub fn get_message<S: ReadonlyStorage>(
    storage: &S,
    for_address: &HumanAddr,
    position: u32
) -> StdResult<Message> {

    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, for_address.0.as_bytes()],
        storage
    );

    // Try to access the storage of files for the account.
    // If it doesn't exist yet, return a Message with path called Empty 
    let store = AppendStore::<Message, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok(Message::new(String::from("Empty/"), String::from("None")))
    };

    store.get_at(position)
} 

pub fn get_messages<S: ReadonlyStorage>(
    storage: &S,
    behalf: &HumanAddr,

) -> StdResult<(Vec<Message>, u64)> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_MSGS_RECEIVED, behalf.0.as_bytes()],
        storage
    );

    // Try to access the collection for the account.
    // If it doesn't exist yet, return an empty collection.
    let store = AppendStore::<Message, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], 0));
    };
    
    let tx_iter = store
        .iter()
        .take(store.len().try_into().unwrap());

    let txs: StdResult<Vec<Message>> = tx_iter
        .map(|tx| tx)
        .collect();
        txs.map(|txs| (txs, store.len() as u64)) //the length of the collection of messages is also returned -- do we need it?
}