extends Spatial


onready var world = $"../../VoxelWorld"
onready var player = $".."


func _ready():
#	world.load_distance = 1
	world.set_voxel(Vector3(3,3,3), 1)
	pass


func _process(_delta):
	world.player_pos = player.translation
#	if Input.is_action_just_pressed("place"):
	if Input.is_action_pressed("place"):
		var hit: Vector3 = world.cast_ray(player.translation, forward())
		hit -= forward()
#		print("hit at ", hit)
		if hit.length() > 0:
			world.set_voxel(hit, 1)
	if Input.is_action_pressed("break"):
		var hit: Vector3 = world.cast_ray(player.translation, forward())
		if hit.length() > 0:
			world.set_voxel(hit, 0)


func forward():
	return (global_transform.origin - player.translation).normalized()
