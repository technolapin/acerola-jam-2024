#version 330 core

layout (triangles) in;
layout(triangle_strip, max_vertices=3) out;

in int vlabel[];
out vec2 fpos;
flat out ivec3 vertices_labels;
flat out mat3x2 vertices_poses;


void main()
{
    vec4 pos0 =  gl_in[0].gl_Position;
    vec4 pos1 =  gl_in[1].gl_Position;
    vec4 pos2 =  gl_in[2].gl_Position;
    int lab0 =  vlabel[0];
    int lab1 =  vlabel[1];
    int lab2 =  vlabel[2];
    vertices_poses = mat3x2(pos0.xy/pos0.w,pos1.xy/pos1.w,pos2.xy/pos2.w);
    vertices_labels = ivec3(lab0,lab1,lab2);

    gl_Position = pos0;
    fpos = pos0.xy/pos0.w; 
    EmitVertex();

    gl_Position = pos1; 
    fpos = pos1.xy/pos1.w; 
    EmitVertex();

    gl_Position = pos2;
    fpos = pos2.xy/pos2.w; 
    EmitVertex();

    EndPrimitive(); 
}
