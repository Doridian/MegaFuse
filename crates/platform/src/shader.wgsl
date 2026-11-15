// The picture, scaled into the window.

struct View {
    // Top left of the picture in the window, in pixels.
    offset: vec2<f32>,
    // Window pixels per picture pixel.
    scale: f32,
    _pad: f32,
};

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var<uniform> view: View;

@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    // One triangle that covers the window.
    let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    return vec4<f32>(uv * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(picture));
    let p = floor((pos.xy - view.offset) / view.scale);
    if (p.x < 0.0 || p.y < 0.0 || p.x >= size.x || p.y >= size.y) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return textureLoad(picture, vec2<i32>(p), 0);
}
