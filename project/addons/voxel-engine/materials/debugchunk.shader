shader_type spatial;


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
	VERTEX = floor(VERTEX);
}

void fragment() {
	// vec3 pos = (CAMERA_MATRIX * vec4(VERTEX, 1.0)).xyz;
	// pos = fract(pos);
	// vec3 normal = (CAMERA_MATRIX * vec4(NORMAL, 0.0)).xyz;

	float uv_vis = 1.0;
	if (UV2.x > 0.98 ||
		UV2.x < 0.02 ||
		UV2.y > 0.98 ||
		UV2.y < 0.02 ||
		abs(UV2.x - UV2.y) < 0.02) {
		uv_vis = 0.5;
	}
	ALBEDO = vec3(uv_vis);
}
