use howto::maths::circumcenter;

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

/** used for the rendering and placement of anything that's in-between the voronoi cells */
#[derive(Debug, Clone)]
pub struct Corner
{
    pub pos: glam::Vec2,
    pub nodes: [usize; 3]
}

/** used for the cellular automata */
#[derive(Debug, Clone, Copy)]
pub struct Adjacency
{
    nodes: [usize; 2],
    magnitude: f32
}


/**
The voronoi diagram is kind of the dual of a delaunay triangulation.
We model the delaunay triangulation with simplicial complexes.

|         | delaunay triangulation | voronoi diagram         |
|---------+------------------------+-------------------------|
| 1-faces | centroids              | cell                    |
| 2-faces | edges                  | cells adjacencies       |
| 3-faces | triangle               | vertices of the diagram  |

It is important to store all of this data.

When we want to draw the voronoi diagram, we need the vertices of the diagram, and we need to be able to access them using their centroids.
When we want to apply some cellular automata logic to the diagram, we need the cell adjacency relationship as well as the edges lengths, as well as the cendroids/cells of the voronoi diagram.
When we want to draw things in between the cells, we need the vertices of the diagram.
Therefore we will store:
+ The centroids (called nodes)
+ The vertices of the diagram (their positions and the 3 centroids they are adjacent to). (called corners)
+ The adjacencies relationship as well with their magnitude (the length of the edge of the voronoi diagram). (called adjacencies)
The adjacencies can be deduced from the vertices of the diagram, we just need to find every pair of vertices sharing the two same centroid adjacencies.
This is expensive, unless we are very smart. And we are particulary dumb, therefore these adjacencies will be stored.
Everything should be calculated from the the centroids of the voronoi diagram.

We also want several level of simulation and rendering resolution. We aim for two levels of resolution: coarse and fine, but more could be used in the future.
First of all, the fine and coarse diagram existe independently. The fine one just happens to have more cells.
The inclusion of the fines cells into the coarse one are done as a wrapping. of one into the other.
struct HierarchicVoronoi
{
    diagrams : Vec<Voronoi>,
    hierarchy: AFuckingTreeWhoseNodeValuesAreTheIndicesOfTheCentroidsOfTheDiagrams
}
HierarchicVoronoi will be able to propagate changes to the dyanmic ressources of the coarses cells to their associated finner cells, and vice et versa.
*/
#[derive(Debug, Default)]
pub struct Voronoi
{
    // 1-faces (used to compute everything else)
    pub nodes_values: Vec<Cell>,
    pub nodes_pos: Vec<glam::Vec2>, 

    pub nodes_adjacency: Vec<Adjacency>, // kinda 2-faces
    pub nodes_corner: Vec<Vec<usize>>, // the polygon of each cell (their indices)
    pub corners_pos: Vec<glam::Vec2>, // dual 3-faces
//    pub corners_node: Vec<[usize; 3]> // 3-faces
//    pub triangles: Vec<(usize,usize,usize)>,
}




impl Voronoi
{
    pub fn new() -> Self
    {
	Self::default()
    }
    // fn closest_from_coord(&self, pos0: glam::Vec2) -> Option<usize>
    // {
    // 	self.nodes.iter()
    // 	    .map(| Node{cell, pos, edges} | pos.distance(pos0))
    // 	    .enumerate()
    // 	    .min_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal))
    // 	    .map(|(index, _)| index)
    // }
    pub fn add_cell(&mut self, cell: Cell, pos: glam::Vec2)
    {
	// unimplemnted!();
	// if self.nodes.len() == 0
	// {
	self.nodes_values.push(cell);
	self.nodes_pos.push(pos);
	// }
	// else
	// {
	    
	// }
    }
    pub fn recompute_all(&mut self)
    {
	self.nodes_adjacency.clear();
	self.corners_pos.clear();
	
	self.nodes_corner.clear();
	for _ in 0..self.nodes_pos.len()
	{
	    self.nodes_corner.push(vec![]);
	}

	let mut max_radius = 0.0;
	let points: Vec<delaunator::Point> = self.nodes_pos.iter().map(|pos| delaunator::Point{x: pos.x as f64, y:pos.y as f64}).collect();
	let now = std::time::Instant::now();
	let result = delaunator::triangulate(&points);
	let elapsed = now.elapsed();
	println!("Retriangulated the graph in {}.{} secs", elapsed.as_secs(), elapsed.subsec_millis());
	let mut edges = std::collections::HashSet::new();
//	let mut circumcenters = Vec::new();
	result.triangles
	    .into_iter()
	    .array_chunks()
	    .for_each(|mut triangle|
		      {
			  triangle.sort();
			  let [i,j,k] = triangle;
//			  self.triangles.push((i,j,k));
			  edges.insert((i,j));
			  edges.insert((i,k));
			  edges.insert((j,k));
			  let indice = self.corners_pos.len();
			  let circ = circumcenter(&self.nodes_pos[i], &self.nodes_pos[j], &self.nodes_pos[k]);
			  let rad = circ.length_squared();
			  // the triangulation produces some very grave triangles at the hull. They should maybe be eventualy filtered out
			  // if rad > max_radius {max_radius = rad;}
			  // let dist = self.nodes_pos[i].distance(circ).max(self.nodes_pos[j].distance(circ)).max(self.nodes_pos[k].distance(circ));
			  // if dist < 0.1
			  // {
			      self.corners_pos.push(circ);
			      self.nodes_corner[i].push(indice);
			      self.nodes_corner[j].push(indice);
			      self.nodes_corner[k].push(indice);
//			  }
		      });
	println!("Max radius of circumcenter: {}", max_radius);
	// now this is getting technical: we sort the indices of the corners relative to any node, using their angle of rotation aroud the centroid
	let angle_arround = |pos:&glam::Vec2, pos0: &glam::Vec2| {(*pos-*pos0).angle_between(glam::Vec2::new(1.0,0.0))};
	for i in 0..self.nodes_corner.len()
	{
	    let cent   = &self.nodes_pos[i];
	    
	    let angles: std::collections::HashMap<usize, f32> = self.nodes_corner[i].clone().into_iter().map(|i| (i, angle_arround(&self.corners_pos[i], cent))).collect();
	    let ord = |i1: &usize,i2: &usize|
	    {
		let a = angles[i1];
		let b = angles[i2];
		a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Less)};
	    self.nodes_corner[i].sort_by(ord);
	}

	
	// at this point the edges are uniques in the hashset
	let mut index = 0;
	for (i,j) in edges.drain()
	{
	    self.nodes_adjacency.push(Adjacency{nodes: [i,j], magnitude: 0.0});
//	    self.nodes[i].edges.push(index);
//	    self.nodes[j].edges.push(index);
	    index+=1;

	}
	    
    }
}



