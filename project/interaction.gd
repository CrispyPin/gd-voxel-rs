extends Spatial


onready var world = $"../../VoxelWorld"

func _ready():
#	world.load_distance = 1
	pass


func _process(_delta):
	world.player_pos = get_parent().translation
