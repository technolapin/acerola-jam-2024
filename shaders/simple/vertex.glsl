#version 330 core

in vec2 pos;
in int label;
flat out int vlabel;

uniform vec2 uScreenSize;
uniform mat4 uProjMatrix;

void main()
{
  vec4 trans = uProjMatrix * vec4(pos, 1.5, 1);
  vlabel = label;
  gl_Position = trans;
}
