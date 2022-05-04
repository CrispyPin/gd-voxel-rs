extends Spatial

export var enable_highlight = true

onready var world = $"/root/Main/VoxelWorld"
onready var player = $".."
onready var indicator = $"/root/Main/HighlightBox"
onready var debugtext = $"/root/Main/DebugUI/DEBUGTEXT"
onready var chunkwire = $"/root/Main/ChunkHighlight"
var vtype := 1

var t_since_update := 0.0

func _ready():
#	world.set_voxel(Vector3(3,3,3), 1)
	pass

func _process(delta):
	chunkwire.translation = (player.translation / 32).floor() * 32
	t_since_update += delta
	if t_since_update >= 0.3:
		world.set_player_pos(player.translation)
		t_since_update = 0
	debugtext.text = " FPS: " + str(Engine.get_frames_per_second())
	debugtext.text += "\n Load distance: " + str(world.load_distance)
	debugtext.text += "\n forward: " + str((forward() * 100).round() / 100)
	debugtext.text += "\n pos: " + str(player.translation.floor())
	debugtext.text += "\n voxel type: " + str(vtype)
	debugtext.text += "\n total chunks: " + str(world.chunk_count())
	debugtext.text += "\n loaded chunks: " + str(world.loaded_chunk_count())
	debugtext.text += "\n empty chunks: " + str(world.empty_chunk_count())
	debugtext.text += "\n generating chunks: " + str(world.waiting_chunk_count())

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
	indicator.translation = (result.pos - result.normal*0.5).floor()
	indicator.visible = result.hit && enable_highlight
	return result


func forward() -> Vector3:
	return (global_transform.origin - player.translation).normalized()
