use macroquad::prelude::*;

const RENDER_W: f32 = 512.0;
const RENDER_H: f32 = 512.0;

fn window_conf() -> Conf {
    Conf {
        window_title: "Platform tile physics test".to_owned(),
        fullscreen: false,
        window_width: RENDER_W as i32,
        window_height: RENDER_H as i32,
        ..Default::default()
    }
}

fn get_screen_camera(width: f32, height: f32) -> Camera2D {
    Camera2D {
        zoom: (vec2(2. / width, 2. / height)),
        target: vec2(width / 2., height / 2.),
        ..Default::default()
    }
}

fn get_camera_for_target(target: &RenderTarget) -> Camera2D {
    let width = target.texture.width() as f32;
    let height = target.texture.height() as f32;
    Camera2D {
        render_target: Some(*target),
        zoom: (vec2(2. / width, 2. / height)),
        target: vec2(width / 2., height / 2.),
        ..Default::default()
    }
}

fn render_pass(src: &RenderTarget, dst: &RenderTarget, material: Material, color: Color) {
    set_camera(&get_camera_for_target(dst));
    gl_use_material(material);
    draw_texture_ex(
        src.texture,
        0.,
        0.,
        color,
        DrawTextureParams {
            dest_size: Some(vec2(RENDER_W as f32, RENDER_H as f32)),
            ..Default::default()
        },
    );
}

fn encode_param(n: u32) -> Color {
    Color {
        r: (n as f32) / 256.0,
        ..Color::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let rt_geom = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_init = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_step1 = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_step2 = render_target(RENDER_W as u32, RENDER_H as u32);
    let rt_final = render_target(RENDER_W as u32, RENDER_H as u32);
    rt_geom.texture.set_filter(FilterMode::Nearest);
    rt_init.texture.set_filter(FilterMode::Nearest);
    rt_step1.texture.set_filter(FilterMode::Nearest);
    rt_step2.texture.set_filter(FilterMode::Nearest);
    rt_final.texture.set_filter(FilterMode::Nearest);
    let init_material = load_material(VERTEX_SHADER, FS_INIT, MaterialParams::default()).unwrap();
    let step_material = load_material(VERTEX_SHADER, FS_STEP, MaterialParams::default()).unwrap();
    let final_material = load_material(VERTEX_SHADER, FS_FINAL, MaterialParams::default()).unwrap();

    let mut n = 1;

    loop {
        set_camera(&get_camera_for_target(&rt_geom));
        gl_use_default_material();
        clear_background(BLACK);
        draw_rectangle(10.0, 40.0, 50.0, 20.0, WHITE);
        draw_rectangle(100.0, 75.0, 10.0, 60.0, WHITE);
        draw_rectangle(45.0, 75.0, 10.0, 60.0, WHITE);
        draw_rectangle(20.0, 100.0, 60.0, 10.0, WHITE);
        draw_triangle(vec2(2.0, 0.0), vec2(40.0, 0.0), vec2(40.0, 38.0), WHITE);
        draw_poly(384.0, 384.0, 7, 80.0, 0.0, WHITE);
        draw_text_ex(
            &format!("{}: hello world from macroquad and the JFA", n),
            50.0,
            20.0,
            TextParams {
                font_size: 18,
                font_scale: -1.0, // macroquad renders text assuming that (0,0) is top left
                font_scale_aspect: -1.0,
                ..Default::default()
            },
        );

        render_pass(&rt_geom, &rt_init, init_material, WHITE);
        render_pass(&rt_init, &rt_step1, step_material, encode_param(32));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(16));
        render_pass(&rt_step2, &rt_step1, step_material, encode_param(8));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(4));
        render_pass(&rt_step2, &rt_step1, step_material, encode_param(2));
        render_pass(&rt_step1, &rt_step2, step_material, encode_param(1));
        let p = 16 + (((n % 40) as f32 / 40.0 * std::f32::consts::PI).sin() * 16.0) as u32;
        render_pass(&rt_step2, &rt_final, final_material, encode_param(p));

        set_camera(&get_screen_camera(RENDER_W as f32, RENDER_H as f32));
        gl_use_default_material();
        draw_texture_ex(
            rt_final.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(RENDER_W as f32, RENDER_H as f32)),
                ..Default::default()
            },
        );

        next_frame().await;
        n += 1;
    }
}

const FS_INIT: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    return vec4((fc - 0.5) / 256.0, 0.0, 1.0);
}
void main() {
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    // if the red channel is over the intensity threshold then count it as a seed
    if (res.r > 0.3) {
        gl_FragColor = pack(gl_FragCoord.xy);
    } else {
        gl_FragColor = vec4(1.0);
    }
}
"#;

const FS_STEP: &'static str = r#"#version 100
precision lowp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec4 pack(vec2 fc) {
    return vec4((fc - 0.5) / 256.0, 0.0, 1.0);
}
vec2 unpack(vec4 t) {
    return vec2(round(t.r * 256.0), round(t.g * 256.0)) + 0.5;
}
void main() {
    vec2 current_pos;
    float current_dist;
    current_pos = unpack(texture2D(Texture, uv));
    current_dist = length(gl_FragCoord.xy - current_pos);
    int r = int(color.r * 256.0);
    vec2 size = vec2(textureSize(Texture, 0));
    for (int dx = -1; dx <= 1; dx += 1) {
        for (int dy = -1; dy <= 1; dy += 1) {
            vec2 newFragCoord = gl_FragCoord.xy + vec2(float(dx * r), float(dy * r));
            vec2 other_pos = unpack(texture2D(Texture, clamp(newFragCoord / size, 0.0, 1.0)));
            float len = length(gl_FragCoord.xy - other_pos);
            if (len < current_dist) {
                current_dist = len;
                current_pos = other_pos;
            }
        }
    }
    gl_FragColor = pack(current_pos);
}
"#;

const FS_FINAL: &'static str = r#"#version 100
precision highp float;
varying vec4 color;
varying vec2 uv;
uniform sampler2D Texture;
vec2 unpack(vec4 t) {
    return vec2(round(t.r * 256.0), round(t.g * 256.0)) + 0.5;
}
void main() {
    float r = color.r * 256.0;
    float len = length(gl_FragCoord.xy - unpack(texture2D(Texture, uv)));
    if (len == 0.0) {
        gl_FragColor = vec4(1.0);
    } else if (len < r * 0.5) {
        gl_FragColor = vec4(1.0, smoothstep(0.0, r * 0.5, len), 0.0, 1.0);
    } else if (len < r) {
        gl_FragColor = vec4(smoothstep(r, r * 0.5, len), 1.0, 0.0, 1.0);
    }
}
"#;

const VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";
