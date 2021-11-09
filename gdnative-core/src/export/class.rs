use crate::export::user_data::UserData;
use crate::export::ClassBuilder;
use crate::object::ownership::{Shared, ThreadAccess, Unique};
use crate::object::{GodotObject, Instance, Instanciable, TRef};

/// Trait used for describing and initializing a Godot script class.
///
/// This trait is used to provide data and functionality to the
/// "data-part" of the class, such as name, initialization and information
/// about exported properties.
///
/// A derive macro is available for this trait. See documentation on the
/// `NativeClass` macro for detailed usage and examples.
///
/// For exported methods, see the [`NativeClassMethods`] trait.
///
/// [`NativeClassMethods`]: ./trait.NativeClassMethods.html
pub trait NativeClass: Sized + 'static {
    /// Base type of the class.
    ///
    /// In Godot, scripting languages can define "script instances" which can be
    /// attached to objects. Because of the dynamic nature, the intended "inheritance"
    /// is not easily implementable properly.
    ///
    /// Instead, delegation is used and most calls to a Godot object query the script instance
    /// first. This way, some methods can be "overwritten" and new ones can be exposed.
    ///
    /// This only works when using so called "variant calls", since the querying of the script
    /// instance is performed there.
    /// When not using variant calls, any direct(*) calls have to be made to the Godot object
    /// directly.
    ///
    /// The base type describes the "most general" type of object this script class can be
    /// attached to.
    ///
    /// *(\*)*: GDNative enables use of "ptrcall"s, which are wrappers for function pointers.
    /// Those do not do explicit checks for script implementations **unless the method
    /// implementation does**.
    type Base: GodotObject;

    /// User-data wrapper type of the class.
    ///
    /// See module-level documentation on `user_data` for more info.
    type UserData: UserData<Target = Self>;

    /// The name of the class.
    ///
    /// In GDNative+NativeScript many classes can be defined in one dynamic library.
    /// To identify which class has to be used, a library-unique name has to be given.
    fn class_name() -> &'static str;

    /// Function that creates a value of `Self`, used for the script-instance. The default
    /// implementation simply panics.
    ///
    /// This function has a reference to the owner object as a parameter, which can be used to
    /// set state on the owner upon creation or to query values
    ///
    /// It is possible to declare script classes without zero-argument constructors. Instances
    /// of such scripts can only be created from Rust using `Instance::emplace`. See
    /// documentation on `Instance::emplace` for an example.
    #[inline]
    fn init(_owner: TRef<'_, Self::Base, Shared>) -> Self {
        panic!(
            "{} does not have a zero-argument constructor",
            Self::class_name()
        )
    }

    /// Register any exported properties to Godot.
    #[inline]
    fn register_properties(_builder: &ClassBuilder<Self>) {}

    /// Convenience method to create an `Instance<Self, Unique>`. This is a new `Self::Base`
    /// with the script attached.
    ///
    /// If `Self::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    fn new_instance() -> Instance<Self, Unique>
    where
        Self::Base: Instanciable,
    {
        Instance::new()
    }

    /// Convenience method to emplace `self` into an `Instance<Self, Unique>`. This is a new
    /// `Self::Base` with the script attached.
    ///
    /// If `Self::Base` is manually-managed, then the resulting `Instance` must be passed to
    /// the engine or manually freed with `Instance::free`. Otherwise, the base object will be
    /// leaked.
    ///
    /// Must be called after the library is initialized.
    #[inline]
    fn emplace(self) -> Instance<Self, Unique>
    where
        Self::Base: Instanciable,
    {
        Instance::emplace(self)
    }
}

/// Trait used to provide information of Godot-exposed methods of a script class.
pub trait NativeClassMethods: NativeClass {
    /// Function that registers all exposed methods to Godot.
    fn register(builder: &ClassBuilder<Self>);
}

/// Trait for types that can be used as the `owner` arguments of exported methods. This trait
/// is sealed and has no public interface.
///
/// # Safety
///
/// Whenever a NativeScript methods is called, it's assumed that the owner is safe to use.
/// When calling a method that may call non-thread-safe methods on its owner from non-Rust
/// code, the official [thread-safety guidelines][thread-safety] must be followed to prevent
/// undefined behavior.
///
/// [thread-safety]: https://docs.godotengine.org/en/stable/tutorials/threads/thread_safe_apis.html
pub trait OwnerArg<'a, T: GodotObject, Access: ThreadAccess + 'static>: private::Sealed {
    #[doc(hidden)]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self;
}

impl<'a, T> private::Sealed for &'a T where T: GodotObject {}
impl<'a, T, Access> OwnerArg<'a, T, Access> for &'a T
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
    #[inline]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self {
        owner.as_ref()
    }
}

impl<'a, T, Access> private::Sealed for TRef<'a, T, Access>
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
}
impl<'a, T, Access> OwnerArg<'a, T, Access> for TRef<'a, T, Access>
where
    T: GodotObject,
    Access: ThreadAccess + 'static,
{
    #[inline]
    fn from_safe_ref(owner: TRef<'a, T, Access>) -> Self {
        owner
    }
}

mod private {
    pub trait Sealed {}
}