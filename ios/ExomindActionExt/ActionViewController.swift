import UIKit
import MobileCoreServices

class ActionViewController: UIViewController {
    @IBOutlet weak var messageLabel: UILabel!
    fileprivate var cookie: String?

    override func viewDidLoad() {
        super.viewDidLoad()

        if !ExtensionUtils.hasKeychainHasEndpoint() {
            self.messageLabel.text = "You need to open Exomind first and sign in."
            return
        }

        for item: Any in self.extensionContext!.inputItems {
            let inputItem = item as! NSExtensionItem

            for provider: Any in inputItem.attachments! {
                let itemProvider = provider as! NSItemProvider
                itemProvider.loadItem(forTypeIdentifier: "public.url", options: nil, completionHandler: { (data, err) -> Void in
                    if let url = data as? URL {
                        let title = inputItem.attributedTitle?.string ?? inputItem.attributedContentText?.string ?? url.absoluteString
                        ExtensionUtils.createLinkObject(url: url.absoluteString, title: title) {
                            DispatchQueue.main.async {
                                self.messageLabel.text = "Added!!"
                            }
                        }
                    } else {
                        print("Only URL can be added. \(String(describing: self.extensionContext?.inputItems))")
                        self.messageLabel.text = "Only URL can be added. \(String(describing: self.extensionContext?.inputItems))"
                    }
                })
            }
        }
    }

    override func didReceiveMemoryWarning() {
        super.didReceiveMemoryWarning()
        // Dispose of any resources that can be recreated.
    }

    @IBAction func handleOk(_ sender: AnyObject) {
        self.extensionContext!.completeRequest(returningItems: [], completionHandler: nil)
    }
}
