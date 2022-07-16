struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) top_left: vec3<f32>,
    @location(1) bottom_right: vec2<f32>,
    @location(2) tex_top_left: vec2<f32>,
    @location(3) tex_bottom_right: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos: vec2<f32>;
    var left: f32 = in.top_left.x;
    var right: f32 = in.bottom_right.x;
    var top: f32 = in.top_left.y;
    var bottom: f32 = in.bottom_right.y;

    switch(i32(in.vertex_index)) {
        case 0: {
            pos = vec2<f32>(left, top);
            out.tex_coord = in.tex_top_left;
            break;
        }
        case 1: {
            pos = vec2<f32>(right, top);
            out.tex_coord = vec2<f32>(in.tex_bottom_right.x, in.tex_top_left.y);
            break;
        }
        case 2: {
            pos = vec2<f32>(left, bottom);
            out.tex_coord = vec2<f32>(in.tex_top_left.x, in.tex_bottom_right.y);
            break;
        }
        case 3: {
            pos = vec2<f32>(right, bottom);
            out.tex_coord = in.tex_bottom_right;
            break;
        }
        default: {}
    }

    out.clip_position = vec4<f32>(pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tex_coord, 1.0, 1.0);
}