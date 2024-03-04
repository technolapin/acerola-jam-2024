/** uses my brain rotten by geometric algebra in order to compute the circumcenter of a triangle from its 3 points */
pub fn circumcenter(pos1: &glam::Vec2,
		pos2: &glam::Vec2,
		pos3: &glam::Vec2) -> glam::Vec2
{
    // we work in projective algebra
    let p1 = glam::Vec3::new(pos1.x, pos1.y, 1.0);
    let p2 = glam::Vec3::new(pos2.x, pos2.y, 1.0);
    let p3 = glam::Vec3::new(pos3.x, pos3.y, 1.0);
    let v12 = p1 - p2; // directional vector of edges
    let v13 = p1 - p3;
    let o = glam::Vec3::new(0.,0.,1.); // origin
    let n12 = v12.cross(o); // normals vectors of edges
    let n13 = v13.cross(o);
    let c12 = p1 + p2; // center points of edges
    let c13 = p1 + p3;
    // perpendicular bissectors (dual of bivectors, as the cross product is the dual of the wedge product)
    let l12 = c12.cross(n12);
    let l13 = c13.cross(n13);
    // computed intersection (1-vector for the same reasons)
    let circum = l12.cross(l13);

    // the homogeneous coordinate is ought to be non-zero, unless we have realy bad karma
    return glam::Vec2::new(circum.x, circum.y)/circum.z;
}

