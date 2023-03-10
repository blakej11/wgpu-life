struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = position;
    out.tex_coord = tex_coord;
    return out;
}

struct LifeParams {
    width : u32,
    height : u32,
    threshold : f32,
}

@group(0) @binding(0) var texture :  texture_storage_2d<r32float, read>;
@group(0) @binding(1) var<uniform> params : LifeParams;

fn hsv_to_rgb(hsv: vec3<f32>) -> vec3<f32> {
    let h : f32 = hsv.x * 6.0f;
    let s : f32 = hsv.y;
    let v : f32 = hsv.z;

    let w : i32 = i32(h);
    let f : f32 = h - f32(w);
    let p : f32 = v * (1.0f - s);
    let q : f32 = v * (1.0f - (s * f));
    let t : f32 = v * (1.0f - (s * (1.0f - f)));

    var r : f32;
    var g : f32;
    var b : f32;

    switch (w) {
        case 0, 6: { r = v; g = t; b = p; }
        case 1: { r = q; g = v; b = p; }
        case 2: { r = p; g = v; b = t; }
        case 3: { r = p; g = q; b = v; }
        case 4: { r = t; g = p; b = v; }
        case 5: { r = v; g = p; b = q; }
	default: { r = v; g = p; b = q; }
    }

    return vec3<f32>(r, g, b);
}

fn render(val: f32) -> vec4<f32> {
    let thresh : f32 = params.threshold;

    if (val < thresh) {
        return vec4<f32>(0f, 0f, 0f, 0f);
    } else {
        let a: f32 = (val - thresh) / (1.0f - thresh);
        let b: f32 = (1.0f - a) * 0.7f;
        return vec4<f32>(hsv_to_rgb(vec3<f32>(b, 1.0f, 1.0f)), 1.0f);
    	//return vec4<f32>(val, 0.0, 0.0, 1.0);
    }
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var loadCoord: vec2<i32> = vec2<i32>(
        i32(in.tex_coord[0] * f32(params.width)),
        i32(in.tex_coord[1] * f32(params.height))
    );
    var cellValue: f32 = textureLoad(texture, loadCoord).x;
    return render(cellValue);
    //return vec4<f32>(cellValue, 0.0, 0.0, 1.0);
}
