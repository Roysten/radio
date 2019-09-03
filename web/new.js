(function() {
	"use strict";

	const form = document.getElementById("form");
	const inputName = document.getElementById("name");
	const inputUrl = document.getElementById("url");

	function dataSubmitted(xhr, statusCode, response) {
		if (statusCode === 200) {
			window.location = "index.html";
		}
	}

	function formSubmitted(ev) {
		ev.preventDefault();

		const name = inputName.value;
		const url = inputUrl.value;
		const dataObj = { name: name, url: url };

		api.submit("POST", "/stream", dataObj, dataSubmitted);

		return false;
	}

	document.addEventListener(
		"DOMContentLoaded",
		function() {
			form.addEventListener(
				"submit",
				formSubmitted
			);
		}
	);

})();
