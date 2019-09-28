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

//external fun rustLayout(platformNode: PlatformNode, changed: Boolean, l: Int, t: Int, r: Int, b: Int)
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
    fun updateProp(k: String, v: String)
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

interface WiredBaseView {
    fun setPadding(left: Int, top: Int, right: Int, bottom: Int)
    fun setTextSize(size: Float)
    fun getPaddingTop(): Int
    fun getPaddingRight(): Int
    fun getPaddingLeft(): Int
    fun getPaddingBottom(): Int
    fun setX(x: Float)
    fun setY(y: Float)
    fun updateProp(k: String, v: Float) {
        when (k) {
            "text_size" ->  setTextSize(v)
            "pad_left" -> setPadding(v.toInt(), getPaddingTop(), getPaddingRight(), getPaddingBottom())
            "pad_top" -> setPadding(getPaddingLeft(), v.toInt(), getPaddingRight(), getPaddingBottom())
            "pad_right" -> setPadding(getPaddingLeft(), getPaddingTop(), v.toInt(), getPaddingBottom())
            "pad_bottom" -> setPadding(getPaddingLeft(), getPaddingTop(), getPaddingRight(), v.toInt())
            "set_x" -> setX(v)
            "set_y" -> setY(v)
        }
    }
}

class WiredTextView(val mContext: Context): TextView(mContext), WiredBaseView, WiredPlatformView {
    override fun updateProp(k: String, v: String) {
        when (k) {
            "text" ->  text = v as String
        }
    }

    override fun updateProp(k: String, v: Float) {
        super.updateProp(k, v)
    }

    override fun updateProp(k: String, v: RustCallback) {
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
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

    override fun updateProp(k: String, v: Any) {
    }
}

class WiredLinearLayout(val mContext: Context): LinearLayout(mContext), WiredPlatformView {
    override fun updateProp(k: String, v: String) {
        when (k) {
            "orientation" ->  when(v as String) {
                "Vertical" -> orientation = LinearLayout.VERTICAL
                "Horizontal" -> orientation = LinearLayout.HORIZONTAL
            }
        }
    }

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
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
    }
}

class WiredPhysicsLayout(val mContext: Context): PhysicsLinearLayout(mContext), WiredPlatformView {
    override fun updateProp(k: String, v: String) {
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
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
    }
}

class RustCallback {
    val ptr: Long = 0
    external fun call(rustCallback: RustCallback)
}

class WiredButton(mContext: Context): Button(mContext), WiredBaseView, WiredPlatformView {
    override fun updateProp(k: String, v: String) {
        when (k) {
            "text" ->  text = v
        }
    }
    override fun updateProp(k: String, v: RustCallback) {
        when (k) {
            "on_press" -> {
                Log.d("Demo", "REGISTERED ON_PRESS")
                setOnClickListener {
                    Log.d("Demo", "You've pressed it!")
                    v.call(v)
                }
            }
            else -> {
                TODO("not implemented $k") //To change body of created functions use File | Settings | File Templates.
            }
        }
    }

    override fun updateProp(k: String, v: Any) {
        TODO("not implemented") //To change body of created functions use File | Settings | File Templates.
    }

    override fun updateProp(k: String, v: Float) {
        super.updateProp(k, v)
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

//        val factory = WiredViewFactory(this)
//        init(factory, rootView)
//        androidExecutor.run()

        Log.d("fruit", "Finished init")

        something(this, rootView)
    }

    external fun something(ctx: Context, rootView: ViewGroup)
    external fun hello(to: String): String
    external fun init(
        factory: WiredViewFactory,
        rootView: WiredLinearLayout
    )
}
