(function() {
	"use strict";

	const streamList = document.getElementById("stream_list");
	const newStream = document.getElementById("new_stream");
	const nowPlaying = document.getElementById("now_playing");
	const nowPlayingTitle = document.getElementById("now_playing_title");

	function nowPlayingLoaded(xhr, statusCode, payload) {
		if (statusCode === 200) {
			const json = JSON.parse(payload);
			if (json === null) {
				nowPlaying.innerText = "¯\\_(ツ)_/¯";
			} else {
				nowPlaying.innerText = json.name;
			}
			nowPlayingTitle.innerText = "";
		}
	}

	function playlistLoaded(xhr, statusCode, payload) {
		if (statusCode === 200) {
			const json = JSON.parse(payload);
			json.sort(
				function(a, b) {
					return a.name > b.name;
				}
			);
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
		const span = document.createElement("span");

		span.innerText = stream.name;
		span.className = "title";

		a.appendChild(span);
		li.appendChild(a);

		a.id = "stream-" + stream.id;
		a.href = "#";
		a.onclick = function(ev) {
			return streamClicked(ev, a);
		};

		streamList.insertBefore(li, newStream);
	}

	function nowPlayingTitleLoaded(xhr, statusCode, payload) {
		if (statusCode === 200) {
			nowPlayingTitle.innerText = JSON.parse(payload);
		}

		window.setTimeout(
			function() {
				api.request("GET", "/now_playing", nowPlayingTitleLoaded);
			},
			5000
		);
	}

	document.addEventListener(
		"DOMContentLoaded",
		function() {
			api.request("GET", "/playlist",	playlistLoaded);
			api.request("GET", "/stream", nowPlayingLoaded);
			api.request("GET", "/now_playing", nowPlayingTitleLoaded);
		}
	);

})();
