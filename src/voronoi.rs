use howto::maths::circumcenter;
use howto::error;
#[derive(Debug, Clone, Copy)]
pub enum Cell
{
    Air,
    Dirt,
    Stone,
}

// #[derive(Debug, Clone)]
// pub struct Node
// {
//     pub cell: Cell,
//     pub pos: glam::Vec2,
// //    edges: Vec<usize>,
// }

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
    pub nodes_ngh: Vec<Vec<usize>>, // kinda 2-faces
    pub adjacency: Vec<Adjacency>, // kinda 2-faces
    
    pub nodes_corner: Vec<Vec<usize>>, // the polygon of each cell (their indices)
    pub corners_pos: Vec<glam::Vec2>, // dual 3-faces
//    pub corners_node: Vec<[usize; 3]> // 3-faces
//    pub triangles: Vec<(usize,usize,usize)>,
}


// #[derive(Debug)]
// pub struct TreeIterator<'a, T>
// {
//     coord: Vec<usize>,
//     tree: &'a mut Tree<T>
// }


#[derive(Debug, Clone, Copy)]
pub enum TreeNode<T>
{
    Parent{val: T, start: usize, len: usize},
    Leaf{val:T}
}


#[derive(Debug, Clone, Copy)]
pub struct TreeIndex(usize, usize);

/**
Tree is stored layer per layer. When constructing the children of a node, all its siblings are constructed simultenaously.
Therefore the children of a node will be designated by the index of the first one in the children's layer and by the number of children.
*/
#[derive(Debug, Clone)]
pub struct Tree<T>
{
    layers: Vec<Vec<TreeNode<T>>>
}



/**
A tree contains a connex part of the plane.
It has a radius and a center.
Assume we have a mother tree T and we want to get the n-th level of depth of that tree in a radius of r around a point p.
Then we do a depth exploration of T:
+ if dist(p, T.p) > r + T.r => stop
+ else if T.is_leaf || T.level = n => add T to output
+ else for all T_child of T, foo(T_child)
*/
impl<T: Copy> Tree<T>
{
    fn get(&self, coord: TreeIndex) -> Option<&TreeNode<T>>
    {
	let TreeIndex(layer, i) = coord;
	self.layers.get(layer).map(|l| l.get(i)).flatten()
    }
    fn get_mut(&mut self, coord: TreeIndex) -> Option<&mut TreeNode<T>>
    {
	let TreeIndex(layer, i) = coord;
	self.layers.get_mut(layer).map(|l| l.get_mut(i)).flatten()
    }
    fn children(&self, coord: TreeIndex) -> Vec<TreeIndex>
    {
	self.get(coord)
	    .map(|node|
		 match node
		 {
		     TreeNode::<T>::Parent{val, start, len} => {(*start..(start+len)).map(|i| TreeIndex(coord.0+1, i)).collect()},
		     _ => vec![]
		 }).unwrap_or(vec![])
    }
    /** depth-first exploration that stops when a certain condition is met */
    fn conditional_depth_exploration<F>(&self, index: TreeIndex, cond: F) -> Vec<TreeIndex>
    where F: Fn(&TreeIndex, &TreeNode<T>) -> bool
    {
	let mut stack = vec![index];
	let mut output = vec![];
	while let Some(id) = stack.pop()
	{
	    let node = self.get(id).unwrap();
	    match node
	    {
		TreeNode::<T>::Leaf{val} => output.push(id),
		_ =>
		{
		    if cond(&id, node)
		    {
			output.push(id);
		    }
		    else
		    {
			stack.append(&mut self.children(id));
		    }
		}
	    }
	}
	output
    }
    
    fn get_children(&self, node: TreeIndex) -> Vec<&TreeNode<T>>
    {
	unimplemented!();
	
    }
    fn get_nth_level_of_root(&self, root: TreeIndex, max_level: usize) -> Vec<TreeIndex>
    {
	self.conditional_depth_exploration(root, |id: &TreeIndex, _content: &TreeNode<T>| id.0 >= max_level)
    }

    fn create_children(&mut self, leaf: TreeIndex, children: Vec<T>) -> Result<(), error::Error>
    {
	let val = *match self.get(leaf)
	{
	    Some(TreeNode::<T>::Leaf{val}) => Ok(val),
	    _ => error::Error::err("Tried to recreate children of a non-child node")
	}?;
	// add a new layer in case it is needed
	if self.layers.len() == leaf.0+1
	{
	    self.layers.push(vec![]);
	}
	let start = self.layers[leaf.0+1].len();
	let len = children.len();
	*self.get_mut(leaf).unwrap() = TreeNode::<T>::Parent{val, start, len};
	self.layers[leaf.0+1].append(&mut children.into_iter().map(|val| TreeNode::<T>::Leaf{val}).collect());

	Ok(())
    }
    
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox
{
    pos: glam::Vec2,
    rad: f32,
}

#[derive(Debug)]
pub struct HierarchicVoronoi
{
    pub voronois: Vec<Voronoi>,
    pub hierarchy: Tree<(BoundingBox, usize)>,
    //pub hierarchy: HashMap<usize, usize>
}



impl HierarchicVoronoi
{
    fn new(first_layer: Voronoi) -> Self
    {
	let hierarchy = Tree::<(BoundingBox, usize)>
	{
	    layers: vec![(0..first_layer.nodes_pos.len())
			 .map(|index|
			      {
				  TreeNode::Leaf{val: (BoundingBox{pos: first_layer.nodes_pos[index], rad: first_layer.max_radius(index).unwrap()}, index)}
			      }).collect()]
	};
//	let hierarchy = Tree::Node(0, (0..first_layer.nodes_pos.len()).map(|i| Tree::Leaf(i)).collect());
	Self
	{
	    voronois: vec![first_layer],
	    hierarchy: hierarchy
	}
    }

    // I will go to hell for this
    fn subdivide_cell<F>(&mut self, generation_rule: F, index: TreeIndex)
	where F: Fn(((&Cell, &BoundingBox), Vec<&Cell>)) -> Vec<(Cell, glam::Vec2)>
    {
	// we filter out unwanted cases that shouldn't occure anyway
	// we also get the index of the cell in its voronoi layer as well as its "bounding box"
	if let Some(TreeNode::<(BoundingBox, usize)>::Leaf{val: (bounding, i)}) = self.hierarchy.get(index).as_deref()
	{
	    // just for security, we check if the layer we are going to write in exists
	    if index.0+1 > self.voronois.len() {return;} // wtf are you thinking
	    if index.0+1 == self.voronois.len() {self.voronois.push(Voronoi::new());}

	    // getting all the relevant data from the parrent layer to compute the subdivisions
	    let parent_layer: &Voronoi = &self.voronois[index.0];
	    let parent_cell = &parent_layer.nodes_values[*i];
	    let ngh: Vec<&Cell> = parent_layer.nodes_ngh[*i].iter().map(|j| &parent_layer.nodes_values[*j]).collect();
	    // computing the subdivision
	    let children = generation_rule(((parent_cell, &bounding), ngh));
	    // getting a reference to the voronoi layer we will push data in
	    let children_layer: &mut Voronoi = self.voronois.get_mut(index.0+1).unwrap(); // exists

	    // pushing the cells in the children voronoi layer
	    for (cell, pos) in children
	    {
		children_layer.add_cell(cell, pos);
	    }
	    // now we will deal with the hierarchic information
	    // for that we just need the bounding box of each newly generated cell, as well as their voronoi indices
	    let start = children_layer.nodes_pos.len();
	    let len = children.len();
	    let mut children_tree_data = vec![];
	    for i in start..(start+len)
	    {
		let child_box = BoundingBox{pos: children_layer.nodes_pos[i], rad: children_layer.max_radius(i).unwrap()};
		children_tree_data.push((child_box, children_layer.nodes_pos.len()-1));

	    }
	    self.hierarchy.create_children(index, children_tree_data);
	    // at this point we are done
	}
    }
}



impl Voronoi
{
    /// half distance to farthest neighbor of node of given index
    pub fn max_radius(&self, index: usize) -> Option<f32>
    {
	let pos = self.nodes_pos[index];
	self.nodes_ngh.get(index).map(|v| v.iter().map(|i| self.nodes_pos[*i].distance(pos)).max_by(|a, b| a.total_cmp(b))).flatten()
    }
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
    // pub fn ngh(&self, index: usize) -> Vec<&Cell>
    // {
	
    // }
    pub fn iterate_nghs(&self) -> Vec<((&Cell,& glam::Vec2) , Vec<&Cell>)>
    {
	self.nodes_values.iter()
	    .zip(self.nodes_pos.iter())
	    .zip(self.nodes_ngh.iter()
		 .map(|ngh_indices| ngh_indices.iter()
		      .map(|i| &self.nodes_values[*i])
		      .collect::<Vec<&Cell>>()))
	    .collect()
    }
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
	self.adjacency.clear();
	self.nodes_ngh.clear();
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
	self.nodes_ngh = vec![vec![]; self.nodes_pos.len()];
	for (i,j) in edges.drain()
	{
	    self.adjacency.push(Adjacency{nodes: [i,j], magnitude: 0.0});
	    self.nodes_ngh[i].push(j);
	    self.nodes_ngh[j].push(i);
//	    self.nodes[i].edges.push(index);
//	    self.nodes[j].edges.push(index);
	    index+=1;

	}
	    
    }
}



