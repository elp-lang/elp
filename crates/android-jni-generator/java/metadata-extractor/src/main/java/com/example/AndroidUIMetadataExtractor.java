import java.io.*;
import java.util.*;
import java.util.jar.*;
import org.objectweb.asm.*;
import org.objectweb.asm.Opcodes;

/**
 * AndroidUIMetadataExtractor
 *
 * Usage:
 * java -cp asm-9.6.jar;. AndroidUIMetadataExtractor <jar1> [<jar2> ...]
 * <output.json>
 *
 * Requirements:
 * - asm-9.6.jar (or compatible ASM version) must be in classpath.
 *
 * Output:
 * JSON file with metadata for public UI classes, constructors, and methods.
 */
public class AndroidUIMetadataExtractor {

	private static final List<String> UI_PREFIXES = Arrays.asList(
			"android.view.",
			"android.widget.",
			"android.app.",
			"android.graphics.",
			"android.animation.",
			"android.transition.",
			"android.content.res.",
			"androidx.appcompat.widget.",
			"androidx.recyclerview.widget.",
			"androidx.cardview.widget.",
			"androidx.constraintlayout.widget.",
			"androidx.coordinatorlayout.widget.",
			"androidx.drawerlayout.widget.",
			"androidx.fragment.app.",
			"com.google.android.material.button.",
			"com.google.android.material.textfield.",
			"com.google.android.material.navigation.",
			"com.google.android.material.card.",
			"com.google.android.material.slider.",
			"com.google.android.material.progressindicator.");

	public static void main(String[] args) throws Exception {
		if (args.length < 2) {
			System.err.println("Usage: java AndroidUIMetadataExtractor <jar1> [<jar2> ...] <output.json>");
			System.exit(1);
		}
		List<String> jarFiles = new ArrayList<>();
		for (int i = 0; i < args.length - 1; i++) {
			jarFiles.add(args[i]);
		}
		String outputFile = args[args.length - 1];

		Map<String, ClassInfo> results = new TreeMap<>();

		for (String jarFile : jarFiles) {
			processJar(jarFile, results);
		}

		writeJson(outputFile, results);
	}

	private static void processJar(String jarPath, Map<String, ClassInfo> results) throws IOException {
		try (JarFile jar = new JarFile(jarPath)) {
			Enumeration<JarEntry> entries = jar.entries();
			while (entries.hasMoreElements()) {
				JarEntry entry = entries.nextElement();
				if (entry.isDirectory() || !entry.getName().endsWith(".class"))
					continue;
				try (InputStream in = jar.getInputStream(entry)) {
					processClass(in, results);
				} catch (VisitSkipException ignore) {
				}
			}
		}
	}

	private static void processClass(InputStream in, Map<String, ClassInfo> results) throws IOException {
		ClassReader cr = new ClassReader(in);
		cr.accept(new ClassVisitor(Opcodes.ASM9) {
			private String className;
			private String superName;
			private boolean isPublic;
			private boolean isSyntheticOrAnonymous;
			private List<MethodInfo> constructors = new ArrayList<>();
			private List<MethodInfo> methods = new ArrayList<>();

			@Override
			public void visit(int version, int access, String name, String signature, String superName,
					String[] interfaces) {
				className = name.replace('/', '.');
				this.superName = superName == null ? null : superName.replace('/', '.');
				isPublic = (access & Opcodes.ACC_PUBLIC) != 0;
				isSyntheticOrAnonymous = (access & Opcodes.ACC_SYNTHETIC) != 0;

				// Only consider public classes in UI packages
				if (!isPublic || isSyntheticOrAnonymous || !isUiClass(className)) {
					throw new VisitSkipException();
				}
				System.out.println("Processing: " + className);
			}

			@Override
			public MethodVisitor visitMethod(int access, String name, String desc, String signature,
					String[] exceptions) {
				boolean isConstructor = name.equals("<init>");
				boolean isStaticInit = name.equals("<clinit>");
				boolean isPublic = (access & Opcodes.ACC_PUBLIC) != 0;
				boolean isSynthetic = (access & Opcodes.ACC_SYNTHETIC) != 0;
				boolean isBridge = (access & Opcodes.ACC_BRIDGE) != 0;

				if (isStaticInit || isSynthetic || isBridge)
					return null;
				if (!isPublic)
					return null;

				MethodInfo mi = new MethodInfo();
				mi.name = name;
				mi.desc = desc;
				mi.signature = signature == null ? desc : signature;
				mi.isConstructor = isConstructor;

				if (isConstructor) {
					constructors.add(mi);
				} else {
					methods.add(mi);
				}
				return null; // Do not visit code
			}

			@Override
			public void visitEnd() {
				if (constructors.isEmpty() && methods.isEmpty())
					return;
				ClassInfo ci = new ClassInfo();
				ci.className = className;
				ci.superName = superName;
				ci.constructors = constructors;
				ci.methods = methods;
				results.put(className, ci);
			}
		}, ClassReader.SKIP_CODE | ClassReader.SKIP_DEBUG | ClassReader.SKIP_FRAMES);
	}

	private static boolean isUiClass(String className) {
		for (String prefix : UI_PREFIXES) {
			if (className.startsWith(prefix))
				return true;
		}
		return false;
	}

	private static void writeJson(String file, Map<String, ClassInfo> results) throws IOException {
		try (Writer out = new BufferedWriter(new OutputStreamWriter(new FileOutputStream(file), "UTF-8"))) {
			out.write("{\n");
			boolean firstClass = true;
			for (Map.Entry<String, ClassInfo> entry : results.entrySet()) {
				if (!firstClass)
					out.write(",\n");
				firstClass = false;
				writeClassJson(out, entry.getKey(), entry.getValue());
			}
			out.write("\n}\n");
		}
	}

	private static void writeClassJson(Writer out, String key, ClassInfo ci) throws IOException {
		out.write("  " + jsonString(key) + ": {\n");
		out.write("    \"className\": " + jsonString(ci.className) + ",\n");
		out.write("    \"superName\": " + jsonString(ci.superName) + ",\n");
		out.write("    \"constructors\": [\n");
		for (int i = 0; i < ci.constructors.size(); i++) {
			if (i > 0)
				out.write(",\n");
			out.write("      " + ci.constructors.get(i).toJson());
		}
		out.write("\n    ],\n");
		out.write("    \"methods\": [\n");
		for (int i = 0; i < ci.methods.size(); i++) {
			if (i > 0)
				out.write(",\n");
			out.write("      " + ci.methods.get(i).toJson());
		}
		out.write("\n    ]\n");
		out.write("  }");
	}

	private static String jsonString(String s) {
		if (s == null)
			return "null";
		StringBuilder sb = new StringBuilder("\"");
		for (int i = 0; i < s.length(); i++) {
			char c = s.charAt(i);
			switch (c) {
				case '\\':
					sb.append("\\\\");
					break;
				case '"':
					sb.append("\\\"");
					break;
				case '\b':
					sb.append("\\b");
					break;
				case '\f':
					sb.append("\\f");
					break;
				case '\n':
					sb.append("\\n");
					break;
				case '\r':
					sb.append("\\r");
					break;
				case '\t':
					sb.append("\\t");
					break;
				default:
					if (c < 0x20) {
						sb.append(String.format("\\u%04x", (int) c));
					} else {
						sb.append(c);
					}
			}
		}
		sb.append("\"");
		return sb.toString();
	}

	// Helper classes
	private static class ClassInfo {
		String className;
		String superName;
		List<MethodInfo> constructors;
		List<MethodInfo> methods;
	}

	private static class MethodInfo {
		String name;
		String desc;
		String signature;
		boolean isConstructor;

		String toJson() {
			return "{"
					+ "\"name\": " + jsonString(name)
					+ ", \"desc\": " + jsonString(desc)
					+ ", \"signature\": " + jsonString(signature)
					+ ", \"isConstructor\": " + isConstructor
					+ "}";
		}
	}

	private static class VisitSkipException extends RuntimeException {
	}

	// ASM dependency note:
	// Download asm-9.6.jar from
	// https://search.maven.org/artifact/org.ow2.asm/asm/9.6/jar
	// Compile: javac -cp asm-9.6.jar AndroidUIMetadataExtractor.java
	// Run: java -cp asm-9.6.jar;. AndroidUIMetadataExtractor <jar1> ...
	// <output.json>
}
