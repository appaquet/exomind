var _endpoint;
var _auth_token;

function addBookmark(e) {
  e.preventDefault();

  let statusDisplay = document.getElementById('status-display');
  let title = document.getElementById('title').value;
  let url = document.getElementById('url').value;

  if (!_endpoint) {
    statusDisplay.innerHTML = 'No endpoint';
    return;
  }

  if (!_auth_token) {
    statusDisplay.innerHTML = 'No auth token';
    return;
  }

  let exomind = protobuf.roots["exomind-root"].exomind;
  let exocore = protobuf.roots["exomind-root"].exocore;
  let google = protobuf.roots["exomind-root"].google;

  let req = new exocore.index.MutationRequest({
    mutations: [
      new exocore.index.EntityMutation({
        putTrait: new exocore.index.PutTraitMutation({
          trait: new exocore.index.Trait({
            message: new google.protobuf.Any({
              type_url: 'type.googleapis.com/exomind.base.Link',
              value: exomind.base.Link.encode(new exomind.base.Link({
                url: url,
                title: title,
              })).finish(),
            })
          })
        })
      }),
      new exocore.index.EntityMutation({
        putTrait: new exocore.index.PutTraitMutation({
          trait: new exocore.index.Trait({
            message: new google.protobuf.Any({
              type_url: 'type.googleapis.com/exomind.base.CollectionChild',
              value: exomind.base.CollectionChild.encode(new exomind.base.CollectionChild({
                collection: new exocore.index.Reference({
                  entityId: 'inbox',
                }),
                weight: new Date().getTime(),
              })).finish(),
            })
          })
        })
      })
    ],
    commonEntityId: true
  });

  console.log(exocore.index.MutationRequest.encode(req).finish());

  const endpointURL = _endpoint + '/entities/mutate?token=' + _auth_token;
  console.log(endpointURL);

  let xhr = new XMLHttpRequest();
  xhr.open('POST', endpointURL, true);
  xhr.setRequestHeader('Content-Type', 'application/protobuf');
  xhr.send(exocore.index.MutationRequest.encode(req).finish());

  xhr.onreadystatechange = function () {
    if (xhr.readyState == 4) {
      statusDisplay.innerHTML = '';
      if (xhr.status == 200) {
        window.close();
      } else {
        statusDisplay.innerHTML = 'Error saving: ' + xhr.statusText;
      }
    }
  };

  statusDisplay.innerHTML = 'Saving...';
}

function fillForm(pageDetails) {
  document.getElementById('title').value = pageDetails.title;
  document.getElementById('url').value = pageDetails.url;
}

function saveEndpoint() {
  let endpoint = document.getElementById('endpoint').value;
  if (!endpoint) {
    return;
  }

  _endpoint = endpoint;
  chrome.storage.local.set({ endpoint: endpoint });
}

function saveAuthToken() {
  let auth_token = document.getElementById('auth_token').value;
  if (!auth_token) {
    return;
  }

  _auth_token = auth_token;
  chrome.storage.local.set({ auth_token: auth_token });
}


window.addEventListener('load', function (evt) {
  document.getElementById('addbookmark').addEventListener('submit', addBookmark);
  document.getElementById('endpoint').addEventListener('change', saveEndpoint);
  document.getElementById('auth_token').addEventListener('change', saveAuthToken);

  chrome.runtime.getBackgroundPage(function (eventPage) {
    eventPage.getPageDetails(fillForm);
  });

  chrome.storage.local.get(['endpoint'], function (result) {
    if (result.endpoint) {
      _endpoint = result.endpoint;
      document.getElementById('endpoint').value = result.endpoint;
    }
  });

  chrome.storage.local.get(['auth_token'], function (result) {
    if (result.auth_token) {
      _auth_token = result.auth_token;
      document.getElementById('auth_token').value = result.auth_token;
    }
  });
});
