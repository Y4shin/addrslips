
use std::{
    collections::{HashMap, HashSet}
};

use uuid::Uuid;
use rstar::RTree;

use super::{Address, util::LookupPoint};

pub struct AddressDatabase {
    addresses: HashMap<Uuid, Address>,
    street_index: HashMap<Uuid, HashSet<Uuid>>,
    addr_index: HashMap<Uuid, HashMap<String, Uuid>>,
    position_index: RTree<LookupPoint>,
    estimated_flats_index: HashMap<u16, HashSet<Uuid>>,
}

impl AddressDatabase {
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
            street_index: HashMap::new(),
            addr_index: HashMap::new(),
            position_index: RTree::new(),
            estimated_flats_index: HashMap::new(),
        }
    }

    pub fn from_addresses(addresses: Vec<Address>) -> Self {
        let street_index = {
            let mut map: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
            for addr in &addresses {
                if let Some(street_id) = addr.assigned_street_id {
                    map.entry(street_id)
                        .or_insert_with(HashSet::new)
                        .insert(addr.id);
                }
            }
            map
        };
        let addr_index = {
            let mut map: HashMap<Uuid, HashMap<String, Uuid>> = HashMap::new();
            for addr in &addresses {
                map.entry(addr.assigned_street_id.unwrap_or(Uuid::nil()))
                    .or_insert_with(HashMap::new)
                    .insert(addr.house_number.clone(), addr.id);
            }
            map
        };
        let position_index = {
            let points = addresses.iter().map(|addr| {
                LookupPoint {
                    id: addr.id,
                    x: addr.position.x as i32,
                    y: addr.position.y as i32,
                }
            });
            RTree::bulk_load(points.collect())
        };
        let estimated_flats_index = {
            let mut map: HashMap<u16, HashSet<Uuid>> = HashMap::new();
            for addr in addresses.iter().filter_map(|a| {
                a.estimated_flats.map(|flats| (a.id, flats))
            }) {
                map.entry(addr.1)
                    .or_insert_with(HashSet::new)
                    .insert(addr.0);
            }
            map
        };
        let addresses = addresses
            .into_iter()
            .map(|addr| (addr.id, addr))
            .collect::<HashMap<Uuid, Address>>();
        Self {
            addresses,
            street_index,
            addr_index,
            position_index,
            estimated_flats_index,
        }
    }

    pub fn dump_db(&self) -> Vec<Address> {
        self.addresses.values().cloned().collect()
    }

    pub fn remove(&mut self, id: &Uuid) -> Option<Address> {
        if let Some(address) = self.addresses.remove(id) {
            if let Some(street_id) = address.assigned_street_id {
                if let Some(addr_set) = self.street_index.get_mut(&street_id) {
                    addr_set.remove(id);
                    if addr_set.is_empty() {
                        self.street_index.remove(&street_id);
                    }
                }
                if let Some(hn_map) = self.addr_index.get_mut(&street_id) {
                    hn_map.remove(&address.house_number);
                    if hn_map.is_empty() {
                        self.addr_index.remove(&street_id);
                    }
                }
            }
            self.position_index.remove(&LookupPoint {
                id: *id,
                x: address.position.x as i32,
                y: address.position.y as i32,
            });
            if let Some(flats) = address.estimated_flats {
                if let Some(id_set) = self.estimated_flats_index.get_mut(&flats) {
                    id_set.remove(id);
                    if id_set.is_empty() {
                        self.estimated_flats_index.remove(&flats);
                    }
                }
            }
            Some(address)
        } else {
            None
        }
    }

    pub fn insert(&mut self, address: Address) {
        assert!(!self.addresses.contains_key(&address.id));
        self.addr_index
            .entry(address.assigned_street_id.unwrap_or(Uuid::nil()))
            .or_insert_with(HashMap::new)
            .insert(address.house_number.clone(), address.id);
        self.position_index.insert(LookupPoint {
            id: address.id,
            x: address.position.x as i32,
            y: address.position.y as i32,
        });
        if let Some(flats) = address.estimated_flats {
            self.estimated_flats_index
                .entry(flats)
                .or_insert_with(HashSet::new)
                .insert(address.id);
        }
        self.addresses.insert(address.id, address);
    }

    pub fn get_by_id(&self, id: &Uuid) -> Option<&Address> {
        self.addresses.get(id)
    }

    pub fn get_by_street(&self, street: Uuid) -> Option<Address> {
        self.addr_index
            .get(&street)
            .and_then(|id_map| id_map.values().next())
            .and_then(|id| self.addresses.get(id))
            .cloned()
    }

    pub fn get_by_addr(&self, street: Uuid, house_number: &str) -> Option<Address> {
        self.addr_index
            .get(&street)
            .and_then(|id_map| id_map.get(house_number))
            .and_then(|id| self.addresses.get(id))
            .cloned()
    }

    pub fn query_by_estimated_flats(&self, flats: u16) -> Option<HashSet<Uuid>> {
        self.estimated_flats_index.get(&flats).cloned()
    }

    pub fn closest_to(&self, x: i32, y: i32) -> Option<Address> {
        self.position_index
            .nearest_neighbor(&[x, y])
            .and_then(|lp| self.addresses.get(&lp.id))
            .cloned()
    }

    pub fn all_addresses_iter(&self) -> impl Iterator<Item = Address> {
        self.addresses.values().cloned()
    }
}