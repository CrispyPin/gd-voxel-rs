extends Spatial


onready var world = $"/root/Main/VoxelWorld"
onready var player = $".."
onready var indicator = $"/root/Main/VoxelCursor"
onready var debugtext = $"/root/Main/DEBUGTEXT"
var vtype := 1

func _ready():
#	world.set_voxel(Vector3(3,3,3), 1)
	pass

func _process(_delta):
	world.player_pos = player.translation
	debugtext.text = "FPS: " + str(Engine.get_frames_per_second())
	debugtext.text += "\nforward: " + str(forward())
	debugtext.text += "\npos: " + str(player.translation.floor())
	debugtext.text += "\nvoxel type: " + str(vtype)

	var result = raycast()

	if Input.is_action_pressed("place"):
		if result.hit:
			if world.get_voxel(result.pos + result.normal*0.5) == 0:
				world.set_voxel(result.pos + result.normal*0.5, vtype)

	if Input.is_action_pressed("break"):
		if result.hit:
			world.set_voxel(result.pos - result.normal*0.4, 0)

	if Input.is_action_just_released("next_item"):
		vtype = ((vtype + 255) % 255) + 1
	if Input.is_action_just_released("prev_item"):
		vtype = ((vtype-2 + 255) % 255)+1


func raycast():
	var result = world.cast_ray(player.translation, forward(), 12.0)
#	indicator.translation = (result.pos + result.normal*0.5).floor()
	indicator.translation = (result.pos - result.normal*0.5).floor()
#	indicator.translation = result.pos.floor()
	indicator.visible = result.hit
	return result


func forward():
	return (global_transform.origin - player.translation).normalized()
