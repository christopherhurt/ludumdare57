#!/bin/sh

bash compile_shader.sh vert ../src/vert_shader.glsl ../generated/vert_shader.spv
bash compile_shader.sh frag ../src/frag_shader.glsl ../generated/frag_shader.spv
bash compile_shader.sh vert ../src/gui_vert_shader.glsl ../generated/gui_vert_shader.spv
bash compile_shader.sh frag ../src/gui_frag_shader.glsl ../generated/gui_frag_shader.spv
