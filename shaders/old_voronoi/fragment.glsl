#version 330 core
precision mediump float;
in vec2 fpos;
flat in mat3x2 vertices_poses;
flat in ivec3 vertices_labels;
out vec4 color;
uniform vec2 uScreenSize;

const vec3 palette[16] =vec3[](
			       vec3(0, 32.+11., 48.+6.)/255.0,
			       vec3(0*16+7,3*16+6,4*16+2)/255.,
			       vec3(5*16+8,6*16+14,7*16+5)/255.0,
			       vec3(6*16+5,7*16+11,8*16+3)/255.0,
			       vec3(8*16+3,9*16+4,9*16+6)/255.0,
			       vec3(9*16+3,10*16+1,10*16+1)/255.0,
			       vec3(14*16+14,14*16+8,13*16+5)/255.0,
			       vec3(15*16+13,15*16+6,14*16+3)/255.0,
			       vec3(11*16+5,8*16+9,0*16+0)/255.0,
			       vec3(12*16+11,4*16+11,1*16+6)/255.0,
			       vec3(13*16+12,3*16+2,2*16+15)/255.0,
			       vec3(13*16+3,3*16+6,8*16+2)/255.0,
			       vec3(6*16+12,7*16+1,12*16+4)/255.0,
			       vec3(2*16+6,8*16+11,13*16+2)/255.0,
			       vec3(2*16+10,10*16+1,9*16+8)/255.0,
			       vec3(8*16+5,9*16+9,0)/255.0);

void main()
{
  mat3x2 delta = vertices_poses - mat3x2(fpos,fpos,fpos);
  mat3 shame_on_me = transpose(delta) * delta; // very non-optimised lmao
  float d1 = shame_on_me[0][0];
  float d2 = shame_on_me[1][1];
  float d3 = shame_on_me[2][2];
  int i = 0;                
  if (d1 < d2 && d1 < d3)
    {
      i = 0;                                  
    }
  else if (d2 < d3)
    {
      i = 1;
    }
  else
    {
      i = 2;
    }
                
  float d = shame_on_me[i][i];
  int label = vertices_labels[i];
  vec2 frag_pos = gl_FragCoord.xy/uScreenSize;
  float attenuation = exp(-d*d*6.0);
  color = vec4(palette[label]*attenuation, 1.0);
  float d01 = length(cross(vec3(delta[0], 0), vec3(vertices_poses[0] - vertices_poses[1], 0))); 
  float d02 = length(cross(vec3(delta[0], 0), vec3(vertices_poses[0] - vertices_poses[2], 0))); 
  float d12 = length(cross(vec3(delta[1], 0), vec3(vertices_poses[1] - vertices_poses[2], 0)));
  float d_edges = min(d01, min(d02, d12));
  //   if (d < 0.00003) color = vec4(1,1,1,1);
  if (d_edges < 0.00015) color = vec4(1,1,1,0) - color;


}
