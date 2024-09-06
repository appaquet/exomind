import UIKit
import SafariServices
import WebKit

class LinkViewController: UIViewController, EntityTraitView {
    @IBOutlet weak var webView: WKWebView!

    fileprivate var entity: EntityExt!
    fileprivate var trait: AnyTraitInstance!

    func loadEntityTrait(entity: EntityExt, trait: AnyTraitInstance, fullEntity: Bool) {
        self.entity = entity
        self.trait = trait
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        guard let typeInstance = self.trait.typeInstance() else {
            return
        }

        if case let .link(link) = typeInstance, let url = URL(string: link.message.url) {
            let request = URLRequest(url: url)
            self.webView.load(request)
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        if let nav = self.navigationController as? NavigationController {
            nav.resetState()
        }
    }
}
