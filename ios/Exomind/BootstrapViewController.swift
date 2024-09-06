import UIKit
import WebKit
import Exocore

class BootstrapViewController: UIViewController, UITextViewDelegate {
    @IBOutlet weak var errorLabel: UILabel!
    @IBOutlet weak var pinLabel: UILabel!
    @IBOutlet weak var configText: UITextView!

    var disco: Discovery?
    var onDone: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()

        startDiscovery()
        self.refreshNodeConfig()

        KeyboardUtils.sharedInstance.addWillShowObserver(self, selector: #selector(handleKeyboardShown))
    }

    @IBAction func onClose(_ sender: Any) {
        if ExocoreUtils.nodeHasCell {
            self.onDone?()
        } else {
            self.errorLabel.text = "Node needs to join a cell"
        }
    }

    @IBAction func onSave(_ sender: Any) {
        do {
            self.disco = nil
            let configJson = self.configText.text ?? ""
            let config = try Exocore_Core_LocalNodeConfig(jsonString: configJson)
            let node = try LocalNode.from(config: config)
            try ExocoreUtils.saveNode(node: node)
            self.tryBoot()
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }
    
    private func startDiscovery() {
        guard let node = ExocoreUtils.node else {
            self.errorLabel.text = "No node configured, but should had one"
            return
        }

        self.errorLabel.text = nil

        do {
            self.disco = try Discovery()
            self.disco?.joinCell(localNode: node, callback: { [weak self] (stage) in
                guard let this = self else { return }

                DispatchQueue.main.async {
                    switch stage {
                    case .pin(let pin):
                        this.pinLabel.text = formatPin(pin)

                    case .success(let newNode):
                        do {
                            try ExocoreUtils.saveNode(node: newNode)
                            this.tryBoot()
                        } catch {
                            this.errorLabel.text = error.localizedDescription
                        }

                    case .error(let err):
                        this.errorLabel.text = err.localizedDescription
                    }
                }
            })
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }

    private func tryBoot() {
        do {
            try ExocoreUtils.bootNode()
            self.onDone?()
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }

    private func refreshNodeConfig() {
        let config = try? ExocoreUtils.node?.config().jsonString()
        self.configText.text = self.jsonPrettyPrint(config ?? "")
    }

    private func jsonPrettyPrint(_ jsonStr: String) -> String {
        if let data = jsonStr.data(using: .utf8),
           let json = try? JSONSerialization.jsonObject(with: data, options: .mutableContainers),
           let jsonData = try? JSONSerialization.data(withJSONObject: json, options: .prettyPrinted) {
            return String(decoding: jsonData, as: UTF8.self)
        } else {
            return jsonStr
        }
    }

    // Change inset of config text view when keyboard shows up so that bottom can be edited.
    // From https://stackoverflow.com/questions/13161666/how-do-i-scroll-the-uiscrollview-when-the-keyboard-appears
    @objc func handleKeyboardShown(_ notification: Notification) {
        let userInfo: NSDictionary = notification.userInfo! as NSDictionary
        let keyboardInfo = userInfo[UIResponder.keyboardFrameBeginUserInfoKey] as! NSValue
        let keyboardSize = keyboardInfo.cgRectValue.size
        let contentInsets = UIEdgeInsets(top: 0, left: 0, bottom: keyboardSize.height, right: 0)
        self.configText.contentInset = contentInsets
        self.configText.scrollIndicatorInsets = contentInsets
    }

    @objc func handleKeyboardHidden(_ notification: Notification) {
        self.configText.contentInset = .zero
        self.configText.scrollIndicatorInsets = .zero
    }

    @IBAction func onReset(_ sender: Any) {
        do {
            self.disco = nil
            let node = try LocalNode.generate()
            try ExocoreUtils.saveNode(node: node)
            refreshNodeConfig()
            startDiscovery()
        } catch {
            self.errorLabel.text = error.localizedDescription
        }
    }

    deinit {
        print("BootstrapViewController > Deinit")
        KeyboardUtils.sharedInstance.removeObserver(self)
    }
}


private func formatPin(_ pin: UInt32) -> String {
    let strPin = pin.description

    var ret = ""
    for (i, char) in strPin.enumerated() {
        if i == 3 || i == 6 {
            ret += " "
        }
        ret += String(char)
    }
    return ret
}
