package world.syed.way2adb_client

import android.os.Bundle
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import java.io.*
import java.net.Socket
import kotlin.concurrent.thread

const val TCP_HOST = "127.0.0.1"
const val TCP_PORT = 8081

class MainActivity : AppCompatActivity() {
    private var client: TCPClient? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        // Start the TCP connection in a separate background thread
        thread {
            handleTcp()
        }
    }

    fun handleTcp() {
        try {
            val socket = Socket(TCP_HOST, TCP_PORT)
            val outputStream = PrintWriter(socket.getOutputStream(), true)
            val inputStream = BufferedReader(InputStreamReader(socket.getInputStream()))

            runOnUiThread {
                Toast.makeText(applicationContext, "Connected to server at $TCP_HOST:$TCP_PORT", Toast.LENGTH_SHORT).show()
            }

            // Continuously read messages from the server
            while (true) {
                val content = inputStream.readLine()
                if (content != null) {
                    println(content)
                }
            }
        } catch (e: IOException) {
            // Handle network errors (like connection failure)
            runOnUiThread {
                Toast.makeText(applicationContext, "Error connecting to server: ${e.message}", Toast.LENGTH_LONG).show()
            }
        }
    }
}
