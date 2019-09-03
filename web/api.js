const api = (function() {
	"use strict";

	function request(method, url, callback) {
		const xhr = new XMLHttpRequest();

		xhr.open(method, url, true);
		xhr.onreadystatechange = function () {
			if(xhr.readyState === 4) {
				callback(xhr, xhr.status, xhr.responseText);
			}
		};

		xhr.send();
	}

	function submit(method, url, data, callback) {
		const xhr = new XMLHttpRequest();

		xhr.open(method, url, true);
		xhr.onreadystatechange = function () {
			if(xhr.readyState === 4) {
				callback(xhr, xhr.status, xhr.responseText);
			}
		};

		xhr.setRequestHeader("Content-Type", "application/json");
		xhr.send(JSON.stringify(data));
	}

	return {
		"request": request,
		"submit": submit,
	};

})();
