
function getPageDetails(callback) { 
    chrome.tabs.executeScript(null, { file: 'content.js' }); 
    chrome.runtime.onMessage.addListener(function(message)  { 
        callback(message); 
    }); 
}; 

