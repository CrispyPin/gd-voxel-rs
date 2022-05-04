shader_type spatial;
// render_mode depth_draw_alpha_prepass;
//render_mode cull_disabled;
//render_mode unshaded;

uniform sampler2D tex_p_x;
uniform sampler2D tex_n_x;
uniform sampler2D tex_p_y;
uniform sampler2D tex_n_y;
uniform sampler2D tex_p_z;
uniform sampler2D tex_n_z;

uniform float disable_debug_uv;

void vertex() {
	float norm_i = fract(VERTEX.x);
	if (norm_i < .01) {
		NORMAL = vec3(1., 0., 0.);
	}
	else if (norm_i < .02) {
		NORMAL = vec3(-1., 0., 0.);
	}
	else if (norm_i < .03) {
		NORMAL = vec3(0., 1., 0.);
	}
	else if (norm_i < .04) {
		NORMAL = vec3(0., -1., 0.);
	}
	else if (norm_i < .05) {
		NORMAL = vec3(0., 0., 1.);
	}
	else {
		NORMAL = vec3(0., 0., -1.);
	}
	UV2 = fract(VERTEX.zy) * 100.0;
	// UV = UV2;
	// UV = vec2(norm_i, 0.0);
	// COLOR = vec4(fract(VERTEX.x), 0., 0., 1.);
	// NORMAL = vec3(0., 1., 0.);

	VERTEX = floor(VERTEX);
}

void fragment() {
	vec3 cam_pos = CAMERA_MATRIX[3].xyz;
	vec3 pos = (CAMERA_MATRIX * vec4(VERTEX, 1.0)).xyz;
	float dist = length(cam_pos - pos);
	float lod = dist/48.0;
	pos = fract(pos);
	vec3 normal = (CAMERA_MATRIX * vec4(NORMAL, 0.0)).xyz;

	vec4 color = vec4(0.0);
	/* if (normal.x > .9) {
		color = texture(tex_p_x, (1.0 - pos.zy));
	}
	else if (normal.x < -.9) {
		color = texture(tex_n_x, vec2(pos.z, 1.0 - pos.y));
	}
	else if (normal.y > .9) {
		color = texture(tex_p_y, vec2(pos.x, pos.z));
	}
	else if (normal.y < -.9) {
		color = texture(tex_n_y, vec2(1.0 - pos.x, pos.z));
	}
	else if (normal.z > .9) {
		color = texture(tex_p_z, vec2(pos.x, 1.0-pos.y));
	}
	else if (normal.z < -.9) {
		color = texture(tex_n_z, (1.0 - pos.xy));
	} */
	if (normal.x > .9) {
		color = textureLod(tex_p_x, (1.0 - pos.zy), lod);
	}
	else if (normal.x < -.9) {
		color = textureLod(tex_n_x, vec2(pos.z, 1.0 - pos.y), lod);
	}
	else if (normal.y > .9) {
		color = textureLod(tex_p_y, vec2(pos.x, pos.z), lod);
	}
	else if (normal.y < -.9) {
		color = textureLod(tex_n_y, vec2(1.0 - pos.x, pos.z), lod);
	}
	else if (normal.z > .9) {
		color = textureLod(tex_p_z, vec2(pos.x, 1.0-pos.y), lod);
	}
	else if (normal.z < -.9) {
		color = textureLod(tex_n_z, (1.0 - pos.xy), lod);
	}
/* 
	vec2 uv = UV;
	if (normal.x > .9) {
		color = textureLod(tex_p_x, uv, lod);
	}
	else if (normal.x < -.9) {
		color = textureLod(tex_n_x, uv, lod);
	}
	else if (normal.y > .9) {
		color = textureLod(tex_p_y, uv, lod);
	}
	else if (normal.y < -.9) {
		color = textureLod(tex_n_y, uv, lod);
	}
	else if (normal.z > .9) {
		color = textureLod(tex_p_z, uv, lod);
	}
	else if (normal.z < -.9) {
		color = textureLod(tex_n_z, uv, lod);
	}
*/
	float uv_vis = 1.0;
	if (UV2.x > 0.98 ||
		UV2.x < 0.02 ||
		UV2.y > 0.98 ||
		UV2.y < 0.02 ||
		abs(UV2.x - UV2.y) < 0.02) {
		uv_vis = 0.5;
	}
	if (disable_debug_uv < 0.1) {
		uv_vis = 1.0;
	}
	// vec3 col = vec3(color, UV.x*10.0);

	// ALBEDO = col * col * uv_vis;
	// ALBEDO = vec3(UV.x*15.0, 0.4, 0.4);
	ALBEDO = color.rgb * color.rgb * uv_vis;
}
