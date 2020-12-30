import Navigation from "./navigation";

export function setupLinkClickNavigation(fallback) {
    document.addEventListener('click', (e) => {
        var el = e.target;

        // if tagname is not a link, try to go up into the parenthood up 5 levels
        for (var i = 0; el.tagName !== 'A' && i < 5; i++) {
            if (el.parentNode) {
                el = el.parentNode;
            }
        }

        if (el.tagName === 'A') {
            let url = el.getAttribute('href');

            // if it's a local URL, we catch it and send it to navigation
            if (url.startsWith('/') || url.startsWith(window.location.origin) && !el.getAttribute('target')) {
                Navigation.navigate(url);
                e.preventDefault();
                e.stopPropagation();
                return false;
            }

            if (fallback) {
                fallback(e, el);
            }
        }
    });
}