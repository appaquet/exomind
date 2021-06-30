import UIKit
import SwiftyJSON
import FontAwesome_swift
import SnapKit

class RichTextEditor: UIViewController {
    fileprivate weak var delegatedScrollView: UIScrollView?
    fileprivate var webview: RichTextEditorWebView!

    private var webviewCursorPosition = CGPoint(x: 0, y: 0)

    private var keyboardRect = CGRect(x: 0, y: 500, width: 0, height: 0)
    private var accessoryHeight = CGFloat(0)
    private var hasFocus: Bool = false

    convenience init(callback: @escaping (JSON?) -> Void) {
        self.init(nibName: nil, bundle: nil)

        self.webview = RichTextEditorWebView()
        self.webview.initialize(callback)
        self.view = self.webview
        self.webview.onHeightChange = { [weak self] (height) in
            self?.checkScrollPosition()
        }
    }

    override func viewDidLoad() {
        KeyboardUtils.sharedInstance.addWillShowObserver(self, selector: #selector(handleKeyboardWillShow))
        KeyboardUtils.sharedInstance.addShownObserver(self, selector: #selector(handleKeyboardShown))
        KeyboardUtils.sharedInstance.addHiddenObserver(self, selector: #selector(handleKeyboardHidden))
    }

    func setNoScroll(_ onCursorPositionChange: ((CGPoint) -> Void)? = nil) {
        self.webview.scrollView.isScrollEnabled = false
        self.webview.setContentCompressionResistancePriority(UILayoutPriority.defaultHigh, for: .vertical)
        self.webview.onCursorPositionChange = onCursorPositionChange
    }

    func delegateScrollTo(_ delegatedScrollView: UIScrollView) {
        self.delegatedScrollView = delegatedScrollView
        self.setNoScroll { [weak self] (cursorPosition) -> Void in
            guard let this = self else {
                return
            }

            print("New cursor position \(cursorPosition)")
            this.webviewCursorPosition = cursorPosition
            this.checkScrollPosition()
        }
    }

    func setContent(_ content: String) {
        self.webview.setContent(content)
    }

    @objc func handleKeyboardWillShow(_ notification: Notification) {
        if let _ = ((notification as NSNotification).userInfo?[UIResponder.keyboardFrameBeginUserInfoKey] as? NSValue)?.cgRectValue {
            self.replaceKeyboardInputAccessoryView()
        }

        self.webview.checkSize()
        self.hasFocus = true

        // https://stackoverflow.com/questions/31774006/how-to-get-height-of-keyboard#33130819
        if let keyboardFrame: NSValue = notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue {
            let keyboardRectangle = keyboardFrame.cgRectValue
            self.keyboardRect = keyboardRectangle
        }

        self.checkScrollPosition()
    }

    @objc func handleKeyboardShown(_ notification: Notification) {
    }

    @objc func handleKeyboardHidden(_ notification: Notification) {
        self.webview.checkSize()
        self.hasFocus = false
    }

    private func checkScrollPosition() {
        guard let outerScroll = self.delegatedScrollView else {
            return
        }

        if self.webviewCursorPosition.y == 0 {
            // no cursor from webview yet, we wait for it to take decision
            return
        }

        let webviewTop = self.view.frame.minY
        let innerScrollOffset = self.webview.scrollView.contentOffset.y

        // y position of the cursor in the outer scrolled frame
        let cursorFramePosition = (webviewTop + self.webviewCursorPosition.y) - (outerScroll.contentOffset.y + outerScroll.adjustedContentInset.top)
        //print("webviewTop=\(webviewTop) cursor=\(self.webviewCursorPosition.y) outerOffset=\(outerScroll.contentOffset.y) inset=\(outerScroll.adjustedContentInset.top)")

        // y position of the cursor on the screen
        let cursorScreenPosition = cursorFramePosition + outerScroll.adjustedContentInset.top

        // smallest y position that is visible (after nav bar)
        let topVisibleY = outerScroll.adjustedContentInset.top

        // y position of the tab bar on the screen
        let bottomBarY = UIScreen.main.bounds.height - outerScroll.adjustedContentInset.bottom

        // highest y position that is visible (keyboard OR tab bar)
        var bottomVisibleY = self.keyboardRect.minY
        if bottomVisibleY > bottomBarY {
            // keyboard is smaller than bottom bar
            bottomVisibleY = bottomBarY
        }

        //print("RichTextEditor > top=\(topVisibleY) cursor=\(cursorScreenPosition) bottom=\(bottomVisibleY)")

        if innerScrollOffset > 0 {
            // webview's scroll is offset, probably because of the keyboard cursor
            // we apply the offset to the outer scroll view
            print("RichTextEditor > Inner scroll is offset by \(innerScrollOffset). Applying it to external.")
            outerScroll.contentOffset = CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y + innerScrollOffset)
            self.webview.scrollView.contentOffset.y = 0.0

        } else if cursorScreenPosition + 40 < topVisibleY {
            // cursor is bellow top nav bar
            let diff = topVisibleY - cursorScreenPosition + 100
            print("RichTextEditor > Cursor is below nav bar. Scrolling down by \(diff).")
            outerScroll.contentOffset = CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y - diff)

        } else if cursorScreenPosition + 40 > bottomVisibleY {
            // cursor is bellow keyboard or bottom tab bar
            let diff = cursorScreenPosition - bottomVisibleY + 100
            print("RichTextEditor > Cursor is below visible bottom. Scrolling up by \(diff).")
            outerScroll.contentOffset = CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y + diff)
        }
    }

    deinit {
        print("RichTextEditor > Deinit")
        KeyboardUtils.sharedInstance.removeObserver(self)
    }
}

class RichTextEditorWebView: HybridWebView {
    var onCursorPositionChange: ((CGPoint) -> Void)?

    func scrollViewDidScroll(_ scrollView: UIScrollView) {
        // prevent any scroll if it's disabled
        // this may happen if keyboard is shown in a text field
        if (!self.scrollView.isScrollEnabled) {
            scrollView.contentOffset = CGPoint(x: 0, y: 0)
        }
    }

    func initialize(_ callback: @escaping (JSON?) -> Void) {
        self.initialize("html-editor", callback: callback)
        self.scrollView.delegate = self
    }

    override func handleCallbackData(_ json: JSON) {
        if let cursorY = json["cursorY"].number {
            self.onCursorPositionChange?(CGPoint(x: 0, y: cursorY.intValue))
            self.checkSize()
        } else {
            super.handleCallbackData(json)
        }
    }

    func setContent(_ content: String) {
        self.setData(["content": content as AnyObject])
    }
}

extension RichTextEditor {
    // from http://stackoverflow.com/questions/30312525/replace-inputaccessoryview-of-keyboard-in-uiwebview-in-swift

    func addNewAccessoryView(_ oldAccessoryView: UIView) {
        let newAccessoryView = RichTextEditorToolsView(frame: oldAccessoryView.frame)
        newAccessoryView.editor = self
        oldAccessoryView.addSubview(newAccessoryView)

        self.accessoryHeight = oldAccessoryView.frame.height

        // so that we hide the < > controls
        // this is platform specific... but only way easy
        newAccessoryView.backgroundColor = UIColor.systemGray4

        newAccessoryView.snp.makeConstraints { (make) in
            make.left.equalTo(oldAccessoryView.snp.left)
            make.top.equalTo(oldAccessoryView.snp.top).offset(1)
            make.height.equalTo(oldAccessoryView.snp.height)
            make.right.equalTo(oldAccessoryView.snp.right)
        }
    }

    func traverseSubViews(_ vw: UIView) -> UIView {
        if (vw.description.hasPrefix("<UIWebFormAccessory")) {
            return vw
        }
        for subview in vw.subviews as [UIView?] {
            if ((subview?.subviews.count)! > 0) {
                let subvw = self.traverseSubViews(subview!)
                if (subvw.description.hasPrefix("<UIWebFormAccessory")) {
                    return subvw
                }
            }
        }
        return UIView()
    }

    func replaceKeyboardInputAccessoryView() {
        // locate accessory view
        let windowCount = UIApplication.shared.windows.count
        if (windowCount < 2) {
            return
        }

        let tempWindow: UIWindow = UIApplication.shared.windows[1] as UIWindow
        let accessoryView: UIView = traverseSubViews(tempWindow)
        if (accessoryView.description.hasPrefix("<UIWebFormAccessory")) {
            // Found the inputAccessoryView UIView
            if (accessoryView.subviews.count > 0) {
                self.addNewAccessoryView(accessoryView)
            }
        }
    }
}

class RichTextEditorToolsView: UIView {
    weak var editor: RichTextEditor!

    let buttons = [
        ("bold", FontAwesome.bold),
        ("strikethrough", FontAwesome.strikethrough),
        ("header-toggle", FontAwesome.heading),
        ("list-ul", FontAwesome.listUl),
        ("list-ol", FontAwesome.listOl),
        ("outdent", FontAwesome.outdent),
        ("indent", FontAwesome.indent),
    ]

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createView()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func createView() {
        let buttonViews = buttons.enumerated().map { (i, tup) -> UIButton in
            let (_, icon) = tup
            let button = UIButton()
            let img = UIImage.fontAwesomeIcon(name: icon, style: .solid, textColor: UIColor.label, size: CGSize(width: 25, height: 25))
            button.tag = i
            button.addTarget(self, action: #selector(handleButtonTouch), for: .touchUpInside)
            button.setImage(img, for: UIControl.State())
            return button
        }

        let stack = UIStackView(arrangedSubviews: buttonViews)
        stack.alignment = .leading
        stack.axis = .horizontal
        stack.spacing = 15
        self.addSubview(stack)
        stack.snp.makeConstraints { (make) in
            make.left.equalTo(self.snp.left).offset(20)
            make.centerY.equalTo(self.snp.centerY)
        }

        let closeButton = UIButton()
        closeButton.setTitle("Done", for: .normal)
        closeButton.setTitleColor(UIColor.label, for: .normal)
        closeButton.addTarget(self, action: #selector(handleCloseKeyboard), for: .touchUpInside)
        self.addSubview(closeButton)
        closeButton.snp.makeConstraints { (make) in
            make.right.equalToSuperview().offset(-20)
            make.centerY.equalToSuperview()
        }
    }

    @objc func handleButtonTouch(_ sender: UIButton) {
        let (name, _) = self.buttons[sender.tag]
        self.editor.webview.setData(["action": name as AnyObject])
    }

    @objc func handleCloseKeyboard(_ sender: UIButton) {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}
