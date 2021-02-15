// thanks https://github.com/kainino0x/wasm-call-js-from-rust/blob/master/html/library.js

mergeInto(LibraryManager.library, {
  get_player_number: function() {
	return document.getElementById("player_number").value;
  },
  start_javascript_worker_thread: function() {
	return start_worker();
  },
  start_javascript_play_sound: function(sound_id) {
	return play_sound(sound_id);
  },
});
