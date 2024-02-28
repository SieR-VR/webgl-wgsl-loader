import { compileWgsl } from "./";


const result = compileWgsl(`
@binding(0) @group(0) var<uniform> frame : u32;
@binding(1) @group(0) var<uniform> resolution : vec2<f32>;
@vertex
fn vtx_main() -> @builtin(position) vec4<f32> {
  let r = sin(f32(frame) / 60.0) * 0.5 + 0.5;
  return vec4f(r, resolution, 1.0);
}

@fragment
fn frag_main(@builtin(position) coord_in: vec4<f32>) -> @location(0) vec4<f32> {
  return vec4(coord_in.xy, 0.0, 1.0);
}
`)

console.log(result.vertexShader);
console.log(result.fragmentShader);
