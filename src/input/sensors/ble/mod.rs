use secure_ble::{connect_to_adapter, connect_to_node, 
authenticate_node, StreamCipher, subscribe, printvalues};

mod node_credentials;
use node_credentials::*;


pub fn init(){
    let central = connect_to_adapter();

    let mut bed_node = connect_to_node(central, node_bed::ADRESS, Duration::from_secs(100)).unwrap();
    authenticate_node(&mut node_bed, node::bed::HMAC_KEY);
    let bed_decryptor = StreamCipher::from_node(&mut sensor_node, node_bed::KEY);
    let bed_fast_rx = subscribe(sensor_node.clone(), node_bed::HMAC_KEY, SLOW_MEASURMENTS_UUID);
    let bed_slow_rx = subscribe(sensor_node.clone(), node_bed::HMAC_KEY, FAST_MEASURMENTS_UUID);

    let mut bed_node = connect_to_node(central, node_bed::ADRESS, Duration::from_secs(100)).unwrap();
    authenticate_node(&mut node_bed, node::bed::HMAC_KEY);
    let bed_decryptor = StreamCipher::from_node(&mut sensor_node, node_bed::KEY);
    let bed_rx = subscribe(sensor_node, node_bed::HMAC_KEY);

}