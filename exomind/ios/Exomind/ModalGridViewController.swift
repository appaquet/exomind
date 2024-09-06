import UIKit

class ModalGridViewController: UIViewController {
    var onClose: (() -> Void)?

    override func viewDidLoad() {
        super.viewDidLoad()
        self.initModalView()
    }

    func initModalView() {
        self.view.backgroundColor = UIColor.black.withAlphaComponent(0.0)
        self.view.isOpaque = true

        UIView.animate(withDuration: 0.2, animations: {
            self.view.backgroundColor = UIColor.black.withAlphaComponent(0.8)
        })

        let closeTapRecogizer = UITapGestureRecognizer(target: self, action: #selector(close))
        self.view.addGestureRecognizer(closeTapRecogizer)
    }

    func showInsideViewController(_ parentVC: UIViewController) {
        parentVC.addChild(self)
        self.view.frame = CGRect(x: 0, y: 0, width: self.view.frame.size.width, height: self.view.frame.size.height);
        self.parent!.view.addSubview(self.view)
    }

    @objc func close() {
        self.removeFromParent()
        self.view.removeFromSuperview()
        self.onClose?()
    }
}
