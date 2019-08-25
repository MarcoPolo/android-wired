package dev.fruit.androiddemo

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.os.*
import android.support.v7.app.AppCompatActivity
import android.util.Log
import android.view.Gravity
import android.view.View
import android.view.ViewGroup
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import com.jawnnypoo.physicslayout.PhysicsLinearLayout
import org.jetbrains.anko.Orientation
import org.jetbrains.anko.doAsync
import org.jetbrains.anko.uiThread
import java.io.File
import java.lang.ref.WeakReference

//pub trait PlatformNode<PropPatch> {
//    type Child: PlatformNode<PropPatch>;
//    /// If you append a child that is attache somewhere else, you should move the child.
//    fn append_child(&self, c: Rc<Self>) -> Result<(), PlatformError>;
//    /// should not tear down the child! since it may be placed somewhere else later
//    fn remove_child(&self, c: &Rc<Self>) -> Result<(), PlatformError>;
//    /// Should not tear down the child (same as remove_child)
//    fn remove_child_index(&self, idx: usize) -> Result<(), PlatformError>;
//    /// Returns the child that was replaced. like mem::replace
//    fn replace_child(&self, old: &Rc<Self>, new: Rc<Self>) -> Result<(), PlatformError>;
//    fn insert_before_index(&self, index: usize, c: Rc<Self>) -> Result<(), PlatformError>;
//    fn parent(&self) -> Result<Weak<Self>, PlatformError>;
//    fn set_parent(&self, parent: Option<Weak<Self>>) -> Result<(), PlatformError>;
//    /// Meta allows passing in extra info like ns if needed
//    fn apply_prop_patch(&self, patch: PropPatch) -> Result<(), PlatformError>;
//    fn key(&self) -> Option<String> {
//        None
//    }
//}

// Interface for PlatformNode
//interface PlatformNode {
//    fun getInnerView(): View
//    fun appendChild(child: PlatformNode)
//    fun removeChild(child: PlatformNode)
//    fun insertBeforeIndex(idx: Int, other: PlatformNode)
//    fun parent(): WeakReference<PlatformNode>?
//    fun setParent(parent: PlatformNode?)
//    fun applyProp(k: String, v: String)
//    fun addEventHandler(event: String, handler: FruitCallback)
//    fun replaceEventHandler(event: String, handler: FruitCallback)
//    fun getMeasuredWidth(): Int
//    fun getMeasuredHeight(): Int
//    fun measure(w: Int, h: Int)
//    fun layout(l: Int, t: Int, r: Int, b: Int)
//}

//external fun rustLayout(platformNode: PlatformNode, changed: Boolean, l: Int, t: Int, r: Int, b: Int)

//class FruitTextView(context: Context) : PlatformNode {
//    override fun layout(l: Int, t: Int, r: Int, b: Int) {
//        mInnerTextView.layout(l, t, r, b)
//    }
//
//    override fun measure(w: Int, h: Int) {
//        mInnerTextView.measure(w, h)
//    }
//
//    override fun getMeasuredHeight(): Int {
//        return mInnerTextView.measuredHeight
//    }
//
//    override fun getMeasuredWidth(): Int {
//        return mInnerTextView.measuredWidth
//    }
//
//    val mInnerTextView: TextView = TextView(context)
//    var mParent: WeakReference<PlatformNode>? = null
//
//    init {
////        mInnerTextView.textSize = 50f
//    }
//
//    override fun getInnerView(): View {
//        return mInnerTextView
//    }
//
//    override fun removeChild(child: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    override fun insertBeforeIndex(idx: Int, other: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    override fun parent(): WeakReference<PlatformNode>? {
//        return mParent
//    }
//
//    override fun setParent(parent: PlatformNode?) {
//        val weak = if (parent == null) { null } else { WeakReference(parent) }
//        this.mParent = weak
//    }
//
//    override fun applyProp(k: String, v: String) {
//        if (k == "text") {
//            mInnerTextView.text = v
//        } else {
//            println("Unhandled prop $k")
//        }
//    }
//
//    override fun appendChild(child: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    override fun addEventHandler(event: String, handler: FruitCallback) {
//        mInnerTextView.setOnClickListener { view ->
//            println("You've pressed the text")
////            handler.callback(NullEvent() as Object)
//        }
//    }
//
//    override fun replaceEventHandler(event: String, handler: FruitCallback) {
//        this.addEventHandler(event, handler);
//    }
//}
//
//class FruitButtonView(context: Context) : PlatformNode {
//    override fun layout(l: Int, t: Int, r: Int, b: Int) {
//        mInnerBtnView.layout(l, t, r, b)
//    }
//
//    val mInnerBtnView: Button = Button(context)
//    var mParent: WeakReference<PlatformNode>? = null
//
//    init {
////        mInnerBtnView.textSize = 50f
//    }
//
//    override fun measure(w: Int, h: Int) {
//        mInnerBtnView.measure(w, h)
//    }
//
//    override fun getMeasuredHeight(): Int {
//        return mInnerBtnView.measuredHeight
//    }
//
//    override fun getMeasuredWidth(): Int {
//        return mInnerBtnView.measuredWidth
//    }
//
//    override fun getInnerView(): View {
//        return mInnerBtnView
//    }
//
//    override fun removeChild(child: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    override fun insertBeforeIndex(idx: Int, other: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    override fun parent(): WeakReference<PlatformNode>? {
//        return mParent
//    }
//
//    override fun setParent(parent: PlatformNode?) {
//        val weak = if (parent == null) { null } else { WeakReference(parent) }
//        this.mParent = weak
//    }
//
//    override fun applyProp(k: String, v: String) {
//        when (k) {
//            "label" -> mInnerBtnView.text = v
//            else -> println("Unhandled prop $k")
//        }
//    }
//
//    override fun appendChild(child: PlatformNode) {
//        Log.e("Fruit", "Not implmented")
//        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
//    }
//
//    // TODO remove event handler
//    override fun addEventHandler(event: String, handler: FruitCallback) {
//        mInnerBtnView.setOnClickListener { view ->
////            handler.callback(NullEvent() as Object)
//        }
//    }
//
//    override fun replaceEventHandler(event: String, handler: FruitCallback) {
//        this.addEventHandler(event, handler);
//    }
//}
//
//class FruitBox(context: Context) : ViewGroup(context), PlatformNode {
//    override fun onLayout(changed: Boolean, l: Int, t: Int, r: Int, b: Int) {
//        Log.d("fruit", "Called onLayout")
////        rustLayout(this, changed, l, t, r, b)
//
//        if (!changed) {
//            return
//        }
//
//        Log.d("fruit", "count is $this.childCount")
////        for (i in 0..this.childCount - 1) {
////            this.getChildAt(i)?.layout(l, t, r, b)
////        }
//    }
//
//    var mParent: WeakReference<PlatformNode>? = null
//
////    init {
////        this.orientation = LinearLayout.VERTICAL
////    }
//
//    override fun getInnerView(): View {
//        return this
//    }
//
//    override fun removeChild(child: PlatformNode) {
//        this.removeView(child.getInnerView())
//    }
//
//    override fun insertBeforeIndex(idx: Int, other: PlatformNode) {
//        this.addView(other.getInnerView(), idx);
//    }
//
//    override fun parent(): WeakReference<PlatformNode>? {
//        return mParent
//    }
//
//    override fun setParent(parent: PlatformNode?) {
//        val weak = if (parent == null) { null } else { WeakReference(parent) }
//        this.mParent = weak
//    }
//
//    override fun applyProp(k: String, v: String) {
//        when (k) {
//            else -> println("Unhandled prop $k")
//        }
//    }
//
//    override fun appendChild(child: PlatformNode) {
//        this.addView(child.getInnerView())
//    }
//
//    // TODO remove event handler
//    override fun addEventHandler(event: String, handler: FruitCallback) {
//        Log.e("Fruit", "No Event Handler for FruitBox")
//    }
//
//    override fun replaceEventHandler(event: String, handler: FruitCallback) {
//        this.addEventHandler(event, handler);
//    }
//}

class NullEvent {}


class FruitCallback {
    val rustPtr: Long = 0
//    external fun callback(data: Object)
}


class AppState(val context: Context, val rootViewGroup: ViewGroup) {
    val platform: Long? = null
}

interface WiredPlatformView {
    fun updateProp(k: String, v: Any)
    fun updateProp(k: String, v: Float)
    fun updateProp(k: String, v: RustCallback)
    fun appendChild(child: WiredPlatformView) {
        addView(child as View)
    }
    fun removeChild(child: WiredPlatformView) {
        removeView(child as View)
    }
    fun removeChildIndex(idx: Int) {
        removeViewAt(idx)
    }
    fun insertChildAt(child: WiredPlatformView, idx: Int) {
        addView(child as View, idx)
    }
    fun addView(child: View)
    fun addView(child: View, idx: Int)
    fun removeView(child: View)
    fun removeViewAt(idx: Int)
}

class WiredTextView(val mContext: Context): TextView(mContext), WiredPlatformView {
    override fun addView(child: View) {
        throw Error("Undefined")
    }

    override fun addView(child: View, idx: Int) {
        throw Error("Undefined")
    }

    override fun removeView(child: View) {
        throw Error("Undefined")
    }

    override fun removeViewAt(idx: Int) {
        throw Error("Undefined")
    }

    override fun updateProp(k: String, v: Any) {
        when (k) {
            "text" ->  text = v as String
        }
    }

    override fun updateProp(k: String, v: Float) {
        when (k) {
            "text_size" -> textSize = v
            "pad_left" -> setPadding(v.toInt(), paddingTop, paddingRight, paddingBottom)
            "pad_top" -> setPadding(paddingLeft, v.toInt(), paddingRight, paddingBottom)
            "set_x" -> x = v
            "set_y" -> y = v
        }
    }

    override fun updateProp(k: String, v: RustCallback) {
        when (k) {
            "on_press" -> {
                Log.d("Demo", "REGISTERED ON_PRESS for text")
                setOnClickListener {
                    Log.d("Demo", "You've pressed it!")
                    (v as RustCallback).call(v)
                }
            }
        }
    }
}

class WiredLinearLayout(val mContext: Context): LinearLayout(mContext), WiredPlatformView {
    override fun updateProp(k: String, v: RustCallback) {
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
    }

    override fun updateProp(k: String, v: Float) {
        when (k) {
            "height" -> layoutParams = ViewGroup.LayoutParams(layoutParams.width, v.toInt())
            "width" -> layoutParams = ViewGroup.LayoutParams(v.toInt(), layoutParams.height)
            "set_x" -> x = v
            "set_y" -> y = v
        }
    }

    override fun updateProp(k: String, v: Any) {
        when (k) {
            "orientation" ->  when(v as String) {
                "Vertical" -> orientation = LinearLayout.VERTICAL
                "Horizontal" -> orientation = LinearLayout.HORIZONTAL
            }
        }
    }
}

class WiredPhysicsLayout(val mContext: Context): PhysicsLinearLayout(mContext), WiredPlatformView {
    override fun updateProp(k: String, v: RustCallback) {
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
    }

    override fun updateProp(k: String, v: Float) {
        when (k) {
            "height" -> layoutParams = ViewGroup.LayoutParams(layoutParams.width, v.toInt())
            "width" -> layoutParams = ViewGroup.LayoutParams(v.toInt(), layoutParams.height)
            "set_x" -> x = v
            "set_y" -> y = v
        }
    }

    override fun updateProp(k: String, v: Any) {
        when (k) {
            "orientation" ->  when(v as String) {
                "Vertical" -> orientation = LinearLayout.VERTICAL
                "Horizontal" -> orientation = LinearLayout.HORIZONTAL
            }
            "fling" ->  when(v as String) {
                "on" -> {
                    physics.enableFling()
                    physics.enablePhysics()
                }
                "off" -> orientation = LinearLayout.HORIZONTAL
            }
        }
    }
}

class RustCallback {
    val ptr: Long = 0
    external fun call(rustCallback: RustCallback)
}

class WiredButton(mContext: Context): Button(mContext), WiredPlatformView {
    override fun updateProp(k: String, v: RustCallback) {
        when (k) {
            "on_press" -> {
                Log.d("Demo", "REGISTERED ON_PRESS")
                setOnClickListener {
                    Log.d("Demo", "You've pressed it!")
                    (v as RustCallback).call(v)
                }
            }
            else -> {
                TODO("not implemented $k") //To change body of created functions use File | Settings | File Templates.
            }
        }
    }

    override fun updateProp(k: String, v: Any) {
        when (k) {
            "text" ->  text = v as String
        }
    }

    override fun updateProp(k: String, v: Float) {
        when (k) {
            "text_size" -> textSize = v
            "left_pad" -> setPadding(v.toInt(), paddingTop, paddingRight, paddingBottom)
            "top_pad" -> setPadding(paddingLeft, v.toInt(), paddingRight, paddingBottom)
            "set_x" -> x = v
            "set_y" -> y = v
        }
    }

    override fun addView(child: View) {
        throw Error("Undefined")
    }

    override fun addView(child: View, idx: Int) {
        throw Error("Undefined")
    }

    override fun removeView(child: View) {
        throw Error("Undefined")
    }

    override fun removeViewAt(idx: Int) {
        throw Error("Undefined")
    }
}


class WiredViewFactory(val mContext: Context) {
    fun createTextView(): WiredPlatformView {
        return WiredTextView(mContext)
    }
    fun createBtnView(): WiredPlatformView {
        val b =  WiredButton(mContext)
        b.text = "BUTTON"
        b.layoutParams =
            ViewGroup.LayoutParams(
                ViewGroup.LayoutParams.WRAP_CONTENT,
                ViewGroup.LayoutParams.WRAP_CONTENT
            )
        return b
    }
    fun createStackLayoutView(): WiredPlatformView {
        val l = WiredLinearLayout(mContext)
        l.layoutParams = ViewGroup.LayoutParams(500, 500)
        l.gravity = Gravity.CENTER
        l.orientation = LinearLayout.VERTICAL
        return l
    }

    fun createPhysicsLayout(): WiredPlatformView {
        val l = WiredPhysicsLayout(mContext)
//        val l = WiredLinearLayout(mContext)
//        l.layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
        l.physics.disablePhysics()
//        l.physics.disableFling()
        l.layoutParams = ViewGroup.LayoutParams(500, 500)
//        l.layoutParams = ViewGroup.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT)
//        l.addView(createBtnView() as View)
//        l.physics.enablePhysics()
        l.physics.enableFling()
//        l.setBackgroundColor(Color.BLUE)
//        l.height = 500
//        l.width = 500
        return l
//        return createStackLayoutView()
    }
}

class Executor {
    // This is the ptr for the rust side
    val ptr: Long = 0
    fun run () {
        val self = this
        doAsync {
            recv(self)

            uiThread {
                poll(self)
                run()
            }
        }
    }

    external fun setup(executor: Executor)
    external fun recv(executor: Executor)
    external fun poll(executor: Executor)
}


class MainActivity : AppCompatActivity() {
    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        Log.d("fruit", "Got intent")

    }

    @SuppressLint("SetTextI18n", "UnsafeDynamicallyLoadedCode")
    override fun onCreate(savedInstanceState: Bundle?) {

        // Load the librust library. Manually doing for quick reload
        super.onCreate(savedInstanceState)
//        val libraryPath = "/data/data/${getApplicationInfo().packageName}/lib"
        val libraryPath = applicationInfo.nativeLibraryDir
        val directory = File(libraryPath)
        val files = directory.listFiles()
        files.sortBy { f -> f.lastModified() }
        System.load("$libraryPath/librust.so")

        // Startup the executor
        val androidExecutor = Executor()
        androidExecutor.setup(androidExecutor)

        Log.d("from rust", hello("Worlld!"))

        val rootView = WiredLinearLayout(this)
        setContentView(rootView)

        val factory = WiredViewFactory(this)
        init(factory, rootView)
        androidExecutor.run()

        Log.d("fruit", "Finished init")
    }

    external fun hello(to: String): String
    external fun init(
        factory: WiredViewFactory,
        rootView: WiredLinearLayout
    )
}
