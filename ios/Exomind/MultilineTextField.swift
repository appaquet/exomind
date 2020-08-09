import UIKit

class MultilineTextField: UITextView, UITextViewDelegate {
    var onChange: ((String) -> Void)?
    var onChanged: ((String) -> Void)?

    override init(frame: CGRect, textContainer: NSTextContainer?) {
        super.init(frame: frame, textContainer: textContainer)
        self.isScrollEnabled = false
        self.delegate = self
        self.font = UIFont.systemFont(ofSize: 14)
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func textViewDidChange(_ textView: UITextView) {
        self.onChange?(self.text)
    }

    func textViewDidEndEditing(_ textView: UITextView) {
        self.onChanged?(self.text)
    }
}
