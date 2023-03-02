/// Sample Usage in JavaScript using Axios

const fs = require('fs');
const axios = require('axios');
const FormData = require('form-data');

const upload = async () => {
	try {
		// turn the (local) file into a filestream
		const file = fs.createReadStream('./testsmall.zip');
		const title = 'uploadTest';

		// create the form data
		const form = new FormData();
		form.append('title', title);
		form.append('file', file);

		// request file upload
		console.log("awaiting response....")
		const response = await axios.post(
			'http://localhost:8080/api/games/upload', 
			form, 
			{ headers: { ...form.getHeaders(), }
		})
		console.log("response received!")
		// await response
		if (response.status === 200) {
		    return 'Upload complete';        
		} else {
            console.log(response);
        }
	} catch (err) {
		throw err;
	}
}

upload();