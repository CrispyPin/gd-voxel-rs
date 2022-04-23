shader_type spatial;

uniform sampler2D tex_p_x;
uniform sampler2D tex_n_x;
uniform sampler2D tex_p_y;
uniform sampler2D tex_n_y;
uniform sampler2D tex_p_z;
uniform sampler2D tex_n_z;

void fragment() {
	vec3 normal = (CAMERA_MATRIX * vec4(NORMAL, 0.0)).xyz;
	vec3 pos = (CAMERA_MATRIX * vec4(VERTEX, 1.0)).xyz;
	pos = fract(pos);
	
	vec2 uv_offset = COLOR.xy;
	
	vec4 col_p_x = texture(tex_p_x, (1.0 - pos.zy) / 16.0 + uv_offset);
	vec4 col_n_x = texture(tex_n_x, vec2(pos.z, 1.0 - pos.y) / 16.0 + uv_offset);
	vec4 col_p_y = texture(tex_p_y, vec2(pos.x, pos.z) 				/ 16.0 + uv_offset);
	vec4 col_n_y = texture(tex_n_y, vec2(1.0 - pos.x, pos.z) 				/ 16.0 + uv_offset);
	vec4 col_p_z = texture(tex_p_z, vec2(pos.x, 1.0-pos.y) / 16.0 + uv_offset);
	vec4 col_n_z = texture(tex_n_z, (1.0 - pos.xy) / 16.0 + uv_offset);
	
	
	vec4 color = vec4(0.0);
	color += col_p_x * max(dot(normal, vec3(1., 0., 0.)),0.0);
	color += col_n_x * max(dot(normal, vec3(-1., 0., 0.)),0.0);
	color += col_p_y * max(dot(normal, vec3(0., 1., 0.)),0.0);
	color += col_n_y * max(dot(normal, vec3(0., -1., 0.)),0.0);
	color += col_p_z * max(dot(normal, vec3(0., 0., 1.)),0.0);
	color += col_n_z * max(dot(normal, vec3(0., 0., -1.)),0.0);

	ALBEDO = color.rgb * color.rgb;
}
