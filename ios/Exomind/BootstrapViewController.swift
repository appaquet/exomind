import UIKit
import WebKit

class BootstrapViewController: UIViewController {
    @IBOutlet weak var configText: UITextView!
    @IBOutlet weak var errorText: UILabel!

    var onDone: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()
        self.renderState()
    }

    func renderState() {
        self.configText.text = ExocoreUtils.cellConfig
        self.errorText.text = ExocoreUtils.error
    }

    @IBAction func onSave(_ sender: Any) {
        ExocoreUtils.cellConfig = self.configText.text
        ExocoreUtils.initialize()

        if ExocoreUtils.initialized {
            self.onDone?()
        } else {
            self.renderState()
        }
    }
}
