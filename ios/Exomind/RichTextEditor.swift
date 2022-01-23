import UIKit
import SwiftyJSON
import FontAwesome_swift
import SnapKit
import WebKit

class RichTextEditor: UIViewController {
    fileprivate weak var outerScroll: UIScrollView?
    fileprivate var webview: RichTextEditorWebView!

    private var keyboardRect = CGRect(x: 0, y: 500, width: 0, height: 0)
    private var callback: ((JSON?) -> Void)?
    
    convenience init(callback: @escaping (JSON?) -> Void) {
        self.init(nibName: nil, bundle: nil)

        self.callback = callback

        self.webview = RichTextEditorWebView()
        self.view = self.webview

        self.webview.editor = self
        self.webview.onHeightChange = { [weak self] (height) in
            self?.ensureCursorVisible()
        }
    }

    override func viewDidLoad() {
        KeyboardUtils.sharedInstance.addWillShowObserver(self, selector: #selector(handleKeyboardWillShow))
        KeyboardUtils.sharedInstance.addWillHideObserver(self, selector: #selector(handleKeyboardWillHide))

        self.webview.initialize { [weak self] (json: JSON?) in
            self?.callback?(json)

            // make sure scroll is still fine on every change
            self?.ensureCursorVisible()
        }
    }

    func delegateScrollTo(_ outerScroll: UIScrollView) {
        self.outerScroll = outerScroll
        self.webview.disableScroll { [weak self] in
            self?.ensureCursorVisible()
        }
        self.webview.setContentCompressionResistancePriority(UILayoutPriority.defaultHigh, for: .vertical)
    }

    func setContent(_ content: String) {
        self.webview.setContent(content)
    }

    func blur() {
        self.webview.blurEditor()
    }

    @objc func handleKeyboardWillShow(_ notification: Notification) {
        self.webview.checkSize()

        // https://stackoverflow.com/questions/31774006/how-to-get-height-of-keyboard#33130819
        if let keyboardFrame: NSValue = notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue {
            let keyboardRectangle = keyboardFrame.cgRectValue
            self.keyboardRect = keyboardRectangle
        }

        // change inset of scroll view to accommodate for keyboard
        if let outerScroll = self.outerScroll {
            let contentInsets = UIEdgeInsets(top: 0, left: 0, bottom: self.keyboardRect.height, right: 0)
            outerScroll.contentInset = contentInsets
            outerScroll.scrollIndicatorInsets = contentInsets
        }

        self.ensureCursorVisible()
    }

    @objc func handleKeyboardWillHide(_ notification: Notification) {
        self.webview.checkSize()

        if let outerScroll = self.outerScroll {
            outerScroll.contentInset = .zero
            outerScroll.scrollIndicatorInsets = .zero
        }

        self.blur()
    }

    private func ensureCursorVisible() {
        if self.outerScroll == nil {
            // only supported with outer scroll
            return
        }

        self.webview.getCursorPosition { [weak self] point in
            self?.ensureCursorVisible(webviewCursor: point)
        }
    }

    private func ensureCursorVisible(webviewCursor: CGPoint) {
        guard let outerScroll = self.outerScroll else {
            // only supported with outer scroll
            return
        }

        let webviewTop = self.view.frame.minY

        // y position of the cursor in the scrolled frame (which is offset under the nav bar)
        let cursorFramePosition = (webviewTop + webviewCursor.y) - (outerScroll.contentOffset.y + outerScroll.adjustedContentInset.top)
        print("RichTextEditor > webviewTop=\(webviewTop) webviewCursor=\(webviewCursor.y) outerOffset=\(outerScroll.contentOffset.y) adjInsetTop=\(outerScroll.adjustedContentInset.top) adjInsetBottom=\(outerScroll.adjustedContentInset.bottom)")

        // y position of the cursor on the screen
        let cursorScreenPosition = cursorFramePosition + outerScroll.adjustedContentInset.top

        // smallest y position that is visible (after nav bar)
        let topVisibleY = outerScroll.adjustedContentInset.top

        // y position of the tab bar on the screen (we are adding keyboard height to inset, when it shows, we need to subtract it)
        let bottomBarY = UIScreen.main.bounds.height - (outerScroll.adjustedContentInset.bottom - self.keyboardRect.height)

        // highest y position that is visible (keyboard OR tab bar)
        var bottomVisibleY = self.keyboardRect.minY
        if bottomVisibleY > bottomBarY {
            // keyboard is smaller than bottom bar
            bottomVisibleY = bottomBarY
        }

        if let lastCorrected = self.webview.scrollDelegate?.lastCorrectOffset,
           let consecutiveCount = self.webview.scrollDelegate?.consecutiveCorrectCount,
           consecutiveCount > 1 {
            let offset = lastCorrected.y
            print("RichTextEditor > Fixing offset using wkwebview correction of \(lastCorrected.y): Scrolling by \(offset)")
            outerScroll.setContentOffset(CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y + offset), animated: true)
            self.webview.scrollDelegate?.lastCorrectOffset = nil
            self.webview.scrollDelegate?.consecutiveCorrectCount = 0
            return
        }

        if webviewCursor.y <= 0 {
            print("RichTextEditor > Can't get webview cursor. Bailing out.")
            return
        }

        print("RichTextEditor > top=\(topVisibleY) cursor=\(cursorScreenPosition) bottom=\(bottomVisibleY)")
        if cursorScreenPosition - 20 < topVisibleY {
            // cursor is bellow top nav bar
            let diff = topVisibleY - cursorScreenPosition + 40
            print("RichTextEditor > Cursor is below nav bar. Scrolling down by \(diff).")
            outerScroll.setContentOffset(CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y - diff), animated: false)

        } else if cursorScreenPosition + 40 > bottomVisibleY {
            // cursor is bellow keyboard or bottom tab bar
            let diff = cursorScreenPosition - bottomVisibleY + 40
            print("RichTextEditor > Cursor is below visible bottom. Scrolling up by \(diff).")
            outerScroll.setContentOffset(CGPoint(x: outerScroll.contentOffset.x, y: outerScroll.contentOffset.y + diff), animated: false)
        }
    }

    deinit {
        print("RichTextEditor > Deinit")
        KeyboardUtils.sharedInstance.removeObserver(self)
    }
}

fileprivate class RichTextEditorWebView: HybridWebView, UIScrollViewDelegate {
    fileprivate var scrollDelegate: DisableScrollDelegate?
    fileprivate weak var editor: RichTextEditor?
    
    func initialize(_ callback: @escaping (JSON?) -> Void) {
        self.initialize("html-editor", callback: callback)

        // snippet from https://stackoverflow.com/questions/59767515/incorrect-positioning-of-getboundingclientrect-after-newline-character
        let strScript = """
                            function getCaretClientPosition() {
                                const sel = window.getSelection();
                                if (!sel || sel.rangeCount === 0) {
                                    return { x: 0, y: 0 };
                                }
                                const range = sel.getRangeAt(0);

                                // check if we have client rects
                                const rects = range.getClientRects();
                                if (!rects.length) {
                                    // if not rects, we're probably on a new line.
                                    // we select the node in order to be able to get position
                                    if (range.startContainer && range.collapsed) {
                                    // explicitly select the contents
                                    range.selectNodeContents(range.startContainer);
                                    }
                                }

                                const pos = range.getBoundingClientRect();
                                return { x: pos.left, y: pos.top };
                            };

                        function blurCurrentElement() {
                            var el = document.activeElement;
                            if (el && el.blur) {
                                el.blur();
                            }
                        }
                        """
        let script = WKUserScript(
                    source: strScript,
                    injectionTime: WKUserScriptInjectionTime.atDocumentStart,
                    forMainFrameOnly: true
                )
        self.configuration.userContentController.addUserScript(script)
        
        // Remove keyboard acessories on the right that are usually bold / italic since we have them in our bar
        self.inputAssistantItem.trailingBarButtonGroups = []
    }
    
    override var inputAccessoryView: UIView? {
        guard let editor = self.editor else { return nil }
        
        let view = RichTextEditorToolsView(frame: CGRect(x: 0, y: 0, width: 0, height: 40))
        view.editor = editor
        return view
    }

    func disableScroll(_ correctionCallback: @escaping () -> ()) {
        self.scrollDelegate = DisableScrollDelegate(correctionCallback: correctionCallback)
        self.scrollView.delegate = self.scrollDelegate
        self.scrollView.isScrollEnabled = false
    }

    fileprivate func getCursorPosition(_ callback: @escaping (CGPoint) -> ()) {
        self.evaluateJavaScript("getCaretClientPosition()") { any, error in
            guard error == nil,
                  let dict = any as? Dictionary<String, Any>,
                  let x = dict["x"] as? Double,
                  let y = dict["y"] as? Double else {

                print("RichTextEditor > Unexpected client caret position \(String(describing: any)) \(String(describing: error))")
                return
            }

            callback(CGPoint(x: x, y: y))
        }
    }

    func setContent(_ content: String) {
        self.setData(["content": content as AnyObject])
    }

    fileprivate func blurEditor() {
        self.evaluateJavaScript("blurCurrentElement()")
    }
}

// Used to block scrolling when we disable scrolling on the webview.
// Normal scrolling is disabled by setting `isScrollEnabled` false, but not pinch and
// keyboard zoom.
fileprivate class DisableScrollDelegate: NSObject, UIScrollViewDelegate {
    fileprivate var lastCorrectOffset: CGPoint?
    fileprivate var consecutiveCorrectCount: Int = 0
    private var correctionCallback: () -> ()

    init(correctionCallback: @escaping () -> ()) {
        self.correctionCallback = correctionCallback
    }

    func scrollViewDidScroll(_ scrollView: UIScrollView) {
        if scrollView.contentOffset.y != 0 {
            if self.lastCorrectOffset == scrollView.contentOffset {
                self.consecutiveCorrectCount += 1
            } else {
                self.consecutiveCorrectCount = 0
                self.lastCorrectOffset = scrollView.contentOffset
            }

            scrollView.contentOffset = CGPoint(x: 0, y: 0)

            if self.consecutiveCorrectCount > 1 {
                self.correctionCallback()
            }
        }
    }
}

fileprivate class RichTextEditorToolsView: UIView {
    weak var editor: RichTextEditor!

    let buttons = [
        ("bold", FontAwesome.bold),
        ("header-toggle", FontAwesome.heading),
        ("outdent", FontAwesome.outdent),
        ("indent", FontAwesome.indent),
        ("list-todo", FontAwesome.checkSquare),
        ("list-ul", FontAwesome.listUl),
        ("code", FontAwesome.code),
        ("strikethrough", FontAwesome.strikethrough),
        ("italic", FontAwesome.italic),
        ("list-ol", FontAwesome.listOl),
        ("code-block", FontAwesome.fileCode),
    ]

    override init(frame: CGRect) {
        super.init(frame: frame)
        self.createView()
    }

    required init?(coder aDecoder: NSCoder) {
        super.init(coder: aDecoder)
    }

    func createView() {
        self.backgroundColor = UIColor.systemGray5
        
        let buttonViews = buttons.enumerated().map { (i, tup) -> UIButton in
            let (_, icon) = tup
            let button = UIButton()
            let img = UIImage.fontAwesomeIcon(name: icon, style: .solid, textColor: UIColor.label, size: CGSize(width: 25, height: 25))
            button.tag = i
            button.addTarget(self, action: #selector(handleButtonTouch), for: .touchUpInside)
            button.setImage(img, for: UIControl.State())
            return button
        }
        
        let scroll = UIScrollView()
        scroll.showsHorizontalScrollIndicator = false
        self.addSubview(scroll)
        scroll.snp.makeConstraints { (make) in
            make.left.equalToSuperview().offset(15)
            make.centerY.equalToSuperview()
        }

        let stack = UIStackView(arrangedSubviews: buttonViews)
        scroll.addSubview(stack)
        stack.alignment = .leading
        stack.axis = .horizontal
        stack.spacing = 15
        stack.snp.makeConstraints { (make) in
            make.left.equalToSuperview()
            make.right.equalToSuperview()
            make.centerY.equalToSuperview()
            make.height.equalTo(scroll.snp.height)
        }

        let closeButton = UIButton()
        closeButton.setTitle("Done", for: .normal)
        closeButton.setTitleColor(UIColor.label, for: .normal)
        closeButton.addTarget(self, action: #selector(handleCloseKeyboard), for: .touchUpInside)
        self.addSubview(closeButton)
        closeButton.snp.makeConstraints { (make) in
            make.right.equalToSuperview().offset(-20)
            make.left.equalTo(scroll.snp.right).offset(10)
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
