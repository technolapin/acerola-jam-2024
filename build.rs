
use std::fs;

 
use std::{
    env, error::Error, path::{Path, PathBuf},
};


const ALLOWED_FILES: [&'static str; 3] = ["vertex.glsl", "geometry.glsl", "fragment.glsl"];

// utility function to iterate and filter shader directories
fn directory_iter(path: &Path, want_dir: bool) -> Vec<PathBuf>
{
    let read = fs::read_dir(path).ok();
    println!("cargo:warning={:?}", read);
    match read
    {
	None => vec![],
	Some(lst) =>
	{
	    lst.filter_map(|child| child.ok())
		.map(|child| child.path())
		.filter(|child| ((! child.is_dir()) ^ want_dir))
		.filter(|child| want_dir || ALLOWED_FILES.contains(&child.file_name().unwrap().to_str().unwrap()))
		.collect()
	}
    }
}

fn main() -> Result<(), Box<dyn Error>>
{
//    let out_dir = env::var("OUT_DIR")?;
    
//    let paths = fs::read_dir("./shaders").unwrap();

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("shaders.rs");

    println!("cargo:warning=generating file {}:?", dest_path.display());
    println!("cargo:warning=AAAAAAAAAAA");
    let mut generated = String::new();
    for shader in directory_iter(Path::new("./shaders"), true)
    {
        println!("cargo:warning=shader: {}", shader.display());
	let shader_name = shader.file_stem().unwrap().to_str().unwrap();
	generated.push_str(&format!("pub mod {} {{\n", shader_name));

	let children = directory_iter(shader.as_path(), false);
	println!("cargo:warning={:?}", &children);
	for stage in children
	{
            println!("cargo:warning=     stage: {}", stage.display());
	    let stage_name = stage.file_stem().unwrap().to_str().unwrap();
	    let stage_code = std::fs::read_to_string(&stage).unwrap();
	    generated.push_str(
		&format!(r#"pub const {}: &'static str = "{}";"#, stage_name.to_uppercase(), stage_code));
	    generated.push_str("\n");
	}
	generated.push_str("\n}\n");
	fs::write(
	    &dest_path,
	    &generated).unwrap();

}


//    println!("cargo:rerun-if-changed=build.rs");
    // let dest_path = Path::new(&out_dir).join("long_string.txt");
    // let mut f = BufWriter::new(File::create(&dest_path)?);

    // let long_string = "abc".repeat(100);
    // write!(f, "{}", long_string)?;

    Ok(())
}
