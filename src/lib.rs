pub mod maths;
//use rand::{SeedableRng, Rng};
pub mod shaders { include!(concat!(env!("OUT_DIR"), "/shaders.rs"));}

pub mod error;

mod tests
{
    #[test]
    fn computing_circumcenters()
    {
	let mut error_count = 0;
	use crate::maths;
	let nb_iter = 1000000;
	for _ in 0..nb_iter
	{
	    let truth = glam::Vec2::new(rand::random::<f32>(), rand::random::<f32>());
	    let radius = rand::random::<f32>()*10.+0.5;
	    let angle1 = rand::random::<f32>()*std::f32::consts::PI*2.0;
	    let angle2 = rand::random::<f32>()*std::f32::consts::PI*2.0;
	    let angle3 = rand::random::<f32>()*std::f32::consts::PI*2.0;
	    let p1 = truth+glam::Vec2::new(angle1.cos(), angle1.sin())*radius;
	    let p2 = truth+glam::Vec2::new(angle2.cos(), angle2.sin())*radius;
	    let p3 = truth+glam::Vec2::new(angle3.cos(), angle3.sin())*radius;
	    let circum = maths::circumcenter(&p1,&p2,&p3);
	    let error = circum.distance(truth);
	    if (error > 0.1)
	    {
		error_count +=1;
		println!("error={} (expected {}, got {})", error, truth, circum);
	    }

	}
	println!("Got {} errors ({}%)", error_count, error_count as f32 / nb_iter as f32*100.);
	assert!(error_count == 0);
	
//	assert!(false);
    }
}
