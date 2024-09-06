import Foundation
import SafariServices

class SFSafariHelper {
    static func getViewControllerForURL(_ url: URL) -> UIViewController {
        // By showing the safari controller in a navigation controller fixes the swipe
        // from edge bug: https://forums.developer.apple.com/thread/29048
        let sfVc = SFSafariViewController(url: url)
        sfVc.modalPresentationStyle = .fullScreen
        let navVc = UINavigationController(rootViewController: sfVc)
        navVc.isNavigationBarHidden = true
        return navVc
    }
}
