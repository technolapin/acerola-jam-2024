#version 330 core

in vec2 pos;
in int label;
flat out int vlabel;

void main()
{
  vlabel = label;
  gl_Position = vec4(pos, 0, 1);
}
