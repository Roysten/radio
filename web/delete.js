(function() {
	"use strict";

	const form = document.getElementById("form");
	const buttonDelete = document.getElementById("button-delete");

	function dataSubmitted(xhr, statusCode, response) {
		if (statusCode === 200) {
			window.location = "index.html";
		}
	}

	function formSubmitted(ev) {
		ev.preventDefault();

		const checkboxes = form.querySelectorAll("input[type=checkbox]");
		const promises = [];
		for (let checkbox of checkboxes) {
			const id = checkbox.id.split("-")[1];
			if (checkbox.checked) {
				const promise = new Promise(
					function(resolve, reject) {
						api.request(
							"DELETE",
							"/stream/" + id,
							function(xhr, statusCode, response) {
								if (statusCode === 200) {
									resolve();
								} else {
									reject();
								}
							}
						);
					}
				);
				promises.push(promise);
			}
		}

		Promise.all(promises).then(
			function() {
				form.submit();
			}
		);

		return true;
	}

	function playlistLoaded(xhr, statusCode, responseText) {
		if (statusCode !== 200) {
			return;
		}

		const playlist = JSON.parse(responseText);
		for (let station of playlist) {
			const wrapperDiv = document.createElement("div");

			const id = "station-" + station.id;
			const elemInput = document.createElement("input");
			elemInput.type = "checkbox";
			elemInput.id = id;

			const elemLabel = document.createElement("label");
			elemLabel.htmlFor = id;
			elemLabel.innerText = station.name;

			wrapperDiv.appendChild(elemInput);
			wrapperDiv.appendChild(elemLabel);
			form.insertBefore(wrapperDiv, buttonDelete);
		}
	}

	document.addEventListener(
		"DOMContentLoaded",
		function() {
			form.addEventListener(
				"submit",
				formSubmitted
			);

			api.request(
				"GET",
				"playlist",
				playlistLoaded
			);
		}
	);

})();
