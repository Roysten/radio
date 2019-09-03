(function() {
	"use strict";

	const streamList = document.getElementById("stream_list");
	const newStream = document.getElementById("new_stream");
	const nowPlaying = document.getElementById("now_playing");

	function nowPlayingLoaded(xhr, statusCode, payload) {
		if (statusCode === 200) {
			const json = JSON.parse(payload);
			if (json === null) {
				nowPlaying.innerText = "¯\\_(ツ)_/¯";
			} else {
				nowPlaying.innerText = json.name;
			}
		}
	}

	function playlistLoaded(xhr, statusCode, payload) {
		if (statusCode === 200) {
			const json = JSON.parse(payload);
			for (let i = 0; i < json.length; ++i) {
				createPlaylistItem(i + 1, json[i]);
			}
		}
	}

	function streamClicked(ev, a) {
		const id = a.id.split("-")[1];
		api.request(
			"PUT",
			"/stream/" + id,
			function() {
				api.request("GET", "/stream", nowPlayingLoaded);
			}
		);
	}

	function createPlaylistItem(n, stream) {
		const li = document.createElement("li");
		const a = document.createElement("a");
		const number = document.createElement("span");
		const span = document.createElement("span");

		span.innerText = stream.name;
		span.className = "title";

		number.innerText = n;
		number.className = "number";

		a.appendChild(number);
		a.appendChild(span);
		li.appendChild(a);

		a.id = "stream-" + stream.id;
		a.href = "#";
		a.onclick = function(ev) {
			return streamClicked(ev, a);
		};

		streamList.insertBefore(li, newStream);
	}

	document.addEventListener(
		"DOMContentLoaded",
		function() {
			api.request("GET", "/playlist",	playlistLoaded);
			api.request("GET", "/stream", nowPlayingLoaded);
		}
	);

})();
