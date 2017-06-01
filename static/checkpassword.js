"use strict"
var qualityfield = document.getElementById('passwordquality')
document.getElementById('password').oninput = function() {
	var p = this.value
	var quality = 0
	if (/\W/.test(p)) {
		quality += 1
	}
	if (p.length >= 10) {
		quality += 1
	}
	if (p.length >= 15) {
		quality += 1
	}
	if (p.length >= 20) {
		quality += 1
	}
	// as if
	if (/p+[a4]+[s$5]{2}(w|v)+[o0]+r+d+/i.test(p)) {
		quality = 0
	}
	qualityfield.textContent = 'Hasło jest ' + ['kiepskie', 'średnie', 'dobre', 'bardzo dobre', 'doskonałe'][quality]
}
