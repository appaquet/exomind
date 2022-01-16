import UIKit

class LogsViewController: UIViewController {
    @IBOutlet weak var logTextView: UITextView!

    override func viewDidLoad() {
        super.viewDidLoad()

        if let nav = self.navigationController as? NavigationController {
            nav.resetState()
            nav.setBarActions([NavigationControllerBarAction(icon: .retweet, handler: {
                self.refreshLogs()
                self.logTextView.scrollToBottom()
            })])
        }

        self.refreshLogs()
    }

    override func viewDidAppear(_ animated: Bool) {
        self.logTextView.scrollToBottom()
    }

    func refreshLogs() {
        guard var logs = try? String.init(contentsOfFile: ExocoreUtils.logFile, encoding: .utf8) else {
            return
        }
        if logs.count > 10000 {
            logs = String(logs.suffix(10000))
        }

        self.logTextView.text = logs
    }
}

extension UITextView {
    func scrollToBottom() {
        let textCount: Int = text.count
        guard textCount >= 1 else {
            return
        }
        scrollRangeToVisible(NSRange(location: textCount - 1, length: 1))
    }
}
