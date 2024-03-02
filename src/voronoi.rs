

#[derive(Debug, Clone, Copy)]
pub enum Cell
{
    Air,
    Dirt,
    Stone,
}

#[derive(Debug, Clone)]
pub struct Node
{
    pub cell: Cell,
    pub pos: glam::Vec2,
    edges: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct Edge
{
    nodes: (usize, usize),
//    area: f32,
}

#[derive(Debug, Default)]
pub struct Graph
{
    pub nodes: Vec<Node>,
    edges: Vec<Edge>,
    pub triangles: Vec<(usize,usize,usize)>,
}





impl Graph
{
    pub fn new() -> Self
    {
	Self::default()
    }
    fn closest_from_coord(&self, pos0: glam::Vec2) -> Option<usize>
    {
	self.nodes.iter()
	    .map(| Node{cell, pos, edges} | pos.distance(pos0))
	    .enumerate()
	    .min_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal))
	    .map(|(index, _)| index)
    }
    pub fn add_cell(&mut self, cell: Cell, pos: glam::Vec2)
    {
	// unimplemnted!();
	// if self.nodes.len() == 0
	// {
	self.nodes.push(Node{cell, pos, edges: vec![]});
	// }
	// else
	// {
	    
	// }
    }
    pub fn recompute_all(&mut self)
    {
	let points: Vec<delaunator::Point> = self.nodes.iter().map(|Node{pos, ..}| delaunator::Point{x: pos.x as f64, y:pos.y as f64}).collect();
	let now = std::time::Instant::now();
	let result = delaunator::triangulate(&points);
	let elapsed = now.elapsed();
	println!("Retriangulated the graph in {}.{} secs", elapsed.as_secs(), elapsed.subsec_millis());
	let mut edges = std::collections::HashSet::new();
	result.triangles
	    .into_iter()
	    .array_chunks()
	    .for_each(|mut triangle|
		 {
		     triangle.sort();
		     let [i,j,k] = triangle;
		     self.triangles.push((i,j,k));
		     edges.insert((i,j));
		     edges.insert((i,k));
		     edges.insert((j,k));
		 });

	self.edges.clear();
	for node in self.nodes.iter_mut()
	{
	    node.edges.clear();
	}
	// at this point the edges are uniques in the hashset
	let mut index = 0;
	edges.drain()
	    .for_each(|(i,j)|
		     {
			 self.edges.push(Edge{nodes: (i,j)});
			 self.nodes[i].edges.push(index);
			 self.nodes[j].edges.push(index);
			 index+=1;

		     });
	    
    }
}



