extends Spatial


onready var world = $"../../VoxelWorld"
onready var player = $".."
onready var debug = $"../../DEBUG"
onready var debug2 = $"../../DEBUG2"
onready var debugtext = $"../../DEBUGTEXT"
var update_debug := true
var vtype := 1

func _ready():
	world.set_voxel(Vector3(3,3,3), 1)
	pass


func _process(_delta):
	world.player_pos = player.translation
	debugtext.text = str(Engine.get_frames_per_second())
	debugtext.text += "\n" + str(forward().round())

	var result = world.cast_ray(player.translation, forward(), 32.0)
	if update_debug:
		debug.translation = result.pos#.floor()
		debug2.translation = result.pos + result.normal*0.4
	if Input.is_action_pressed("place"):
		if result.hit:
			world.set_voxel(result.pos + result.normal*0.5, vtype)
			vtype = (vtype % 255) + 1

	if Input.is_action_pressed("break"):
#		var result = world.cast_ray(player.translation, forward())
#		debug.translation = result.pos
#		debug2.translation = result.pos + result.normal
		if result.hit:
			world.set_voxel(result.pos - result.normal*0.4, 0)

	if Input.is_action_just_pressed("f1"):
		update_debug = !update_debug


func forward():
	return (global_transform.origin - player.translation).normalized()
