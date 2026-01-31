use rstar::{AABB, PointDistance, RTreeObject};
use uuid::Uuid;

#[derive(PartialEq, Eq, Clone)]
pub struct LookupPoint {
    pub id: Uuid,
    pub x: i32,
    pub y: i32,
}

impl RTreeObject for LookupPoint {
    type Envelope = AABB<[i32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_point([self.x, self.y])
    }
}

impl PointDistance for LookupPoint {
    fn distance_2(&self, point: &[i32; 2]) -> i32 {
        let dx = self.x - point[0];
        let dy = self.y - point[1];
        dx * dx + dy * dy
    }
}