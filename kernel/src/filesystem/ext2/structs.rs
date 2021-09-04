/*
 * Taken from:
 *
 *  linux/include/linux/ext2_fs.h
 *
 * Copyright (C) 1992, 1993, 1994, 1995
 * Remy Card (card@masi.ibp.fr)
 * Laboratoire MASI - Institut Blaise Pascal
 * Universite Pierre et Marie Curie (Paris VI)
 *
 *  from
 *
 *  linux/include/linux/minix_fs.h
 *
 *  Copyright (C) 1991, 1992  Linus Torvalds
 */

const EXT2_DEFAULT_PREALLOC_BLOCKS: u8 = 8;
////
//// The second extended file system version
////
const EXT2FS_DATE: &str = "95/08/09";
const EXT2FS_VERSION: &str = "0.5b";
////
//// Special inode numbers
////
//// Bad blocks inode
const EXT2_BAD_INO: u8 = 1;
//// Root inode
const EXT2_ROOT_INO: u8 = 2;
//// ACL inode
const EXT2_ACL_IDX_INO: u8 = 3;
//// ACL inode
const EXT2_ACL_DATA_INO: u8 = 4;
//// Boot loader inode
const EXT2_BOOT_LOADER_INO: u8 = 5;
//// Undelete directory inode
const EXT2_UNDEL_DIR_INO: u8 = 6;
//// Reserved group descriptors inode
const EXT2_RESIZE_INO: u8 = 7;
//// Journal inode
const EXT2_JOURNAL_INO: u8 = 8;
//// First non-reserved inode for old ext2 filesystems */
const EXT2_GOOD_OLD_FIRST_INO: u8 = 11;
////
//// The second extended file system magic number
////
const EXT2_SUPER_MAGIC: u16 = 0xEF53;
////
//// Maximal count of links to a file
////
const EXT2_LINK_MAX: u16 = 65000;
////
//// Macro-instructions used to manage several block sizes
////
const EXT2_MIN_BLOCK_LOG_SIZE: u8 = 10;
const EXT2_MAX_BLOCK_LOG_SIZE: u8 = 16;
const EXT2_MIN_BLOCK_SIZE: usize = 1 << EXT2_MIN_BLOCK_LOG_SIZE;
const EXT2_MAX_BLOCK_SIZE: usize = 1 << EXT2_MAX_BLOCK_LOG_SIZE;
////
//// Macro-instructions used to manage fragments
////
const EXT2_MIN_FRAG_SIZE: usize = EXT2_MIN_BLOCK_SIZE;
const EXT2_MAX_FRAG_SIZE: usize = EXT2_MAX_BLOCK_SIZE;
const EXT2_MIN_FRAG_LOG_SIZE: u8 = EXT2_MIN_BLOCK_LOG_SIZE;
//// Inode table/bitmap not initialized
const EXT2_BG_INODE_UNINIT: u16 = 0x0001;
//// Block bitmap not initialized
const EXT2_BG_BLOCK_UNINIT: u16 = 0x0002;
//// On-disk itable initialized to zero
const EXT2_BG_INODE_ZEROED: u16 = 0x0004;
const EXT2_HASH_LEGACY: u8 = 0;
const EXT2_HASH_HALF_MD4: u8 = 1;
const EXT2_HASH_TEA: u8 = 2;
//// reserved for userspace lib
const EXT2_HASH_LEGACY_UNSIGNED: u8 = 3;
//// reserved for userspace lib
const EXT2_HASH_HALF_MD4_UNSIGNED: u8 = 4;
//// reserved for userspace lib
const EXT2_HASH_TEA_UNSIGNED: u8 = 5;
const EXT2_HASH_FLAG_INCOMPAT: u8 = 0x1;
const EXT2_MIN_DESC_SIZE: u8 = 32;
const EXT2_MIN_DESC_SIZE_64BIT: u8 = 64;
const EXT2_MAX_DESC_SIZE: usize = EXT2_MIN_BLOCK_SIZE;
////
//// Constants relative to the data blocks
////
const EXT2_NDIR_BLOCKS: usize = 12;
const EXT2_IND_BLOCK: usize = EXT2_NDIR_BLOCKS;
const EXT2_DIND_BLOCK: usize = EXT2_IND_BLOCK + 1;
const EXT2_TIND_BLOCK: usize = EXT2_DIND_BLOCK + 1;
const EXT2_N_BLOCKS: usize = EXT2_TIND_BLOCK + 1;
////
//// Inode flags
////
//// Secure deletion
const EXT2_SECRM_FL: u32 = 0x00000001;
//// Undelete
const EXT2_UNRM_FL: u32 = 0x00000002;
//// Compress file
const EXT2_COMPR_FL: u32 = 0x00000004;
//// Synchronous updates
const EXT2_SYNC_FL: u32 = 0x00000008;
//// Immutable file
const EXT2_IMMUTABLE_FL: u32 = 0x00000010;
//// writes to file may only append
const EXT2_APPEND_FL: u32 = 0x00000020;
//// do not dump file
const EXT2_NODUMP_FL: u32 = 0x00000040;
//// do not update atime
const EXT2_NOATIME_FL: u32 = 0x00000080;
//// Reserved for compression usage... */
const EXT2_DIRTY_FL: u32 = 0x00000100;
//// One or more compressed clusters
const EXT2_COMPRBLK_FL: u32 = 0x00000200;
//// Access raw compressed data
const EXT2_NOCOMPR_FL: u32 = 0x00000400;
//// Compression error
const EXT2_ECOMPR_FL: u32 = 0x00000800;
//// End compression flags --- maybe not all used */
//// btree format dir
const EXT2_BTREE_FL: u32 = 0x00001000;
//// hash-indexed directory
const EXT2_INDEX_FL: u32 = 0x00001000;
const EXT2_IMAGIC_FL: u32 = 0x00002000;
//// file data should be journaled
const EXT3_JOURNAL_DATA_FL: u32 = 0x00004000;
//// file tail should not be merged
const EXT2_NOTAIL_FL: u32 = 0x00008000;
//// Synchronous directory modifications
const EXT2_DIRSYNC_FL: u32 = 0x00010000;
//// Top of directory hierarchies
const EXT2_TOPDIR_FL: u32 = 0x00020000;
//// Set to each huge file
const EXT4_HUGE_FILE_FL: u32 = 0x00040000;
//// Inode uses extents
const EXT4_EXTENTS_FL: u32 = 0x00080000;
//// reserved for ext2 lib
const EXT2_RESERVED_FL: u32 = 0x80000000;
//// User visible flags
const EXT2_FL_USER_VISIBLE: u32 = 0x000BDFFF;
//// User modifiable flags
const EXT2_FL_USER_MODIFIABLE: u32 = 0x000080FF;
////
//// File system states
////
//// Unmounted cleanly
const EXT2_VALID_FS: u16 = 0x0001;
//// Errors detected
const EXT2_ERROR_FS: u16 = 0x0002;
//// Orphans being recovered
const EXT3_ORPHAN_FS: u16 = 0x0004;
////
//// Misc. filesystem flags
////
//// Signed dirhash in use
const EXT2_FLAGS_SIGNED_HASH: u16 = 0x0001;
//// Unsigned dirhash in use
const EXT2_FLAGS_UNSIGNED_HASH: u16 = 0x0002;
//// OK for use on development code
const EXT2_FLAGS_TEST_FILESYS: u16 = 0x0004;
////
//// Mount flags
////
//// Do mount-time checks
const EXT2_MOUNT_CHECK: u16 = 0x0001;
//// Create files with directory's group
const EXT2_MOUNT_GRPID: u16 = 0x0004;
//// Some debugging messages
const EXT2_MOUNT_DEBUG: u16 = 0x0008;
//// Continue on errors
const EXT2_MOUNT_ERRORS_CONT: u16 = 0x0010;
//// Remount fs ro on errors
const EXT2_MOUNT_ERRORS_RO: u16 = 0x0020;
//// Panic on errors
const EXT2_MOUNT_ERRORS_PANIC: u16 = 0x0040;
//// Mimics the Minix statfs
const EXT2_MOUNT_MINIX_DF: u16 = 0x0080;
//// Disable 32-bit UIDs
const EXT2_MOUNT_NO_UID32: u16 = 0x0200;
////
//// Maximal mount counts between two filesystem checks
////
//// Allow 20 mounts
const EXT2_DFL_MAX_MNT_COUNT: u8 = 20;
//// Don't use interval check
const EXT2_DFL_CHECKINTERVAL: u8 = 0;
////
//// Behaviour when detecting errors
////
//// Continue execution
const EXT2_ERRORS_CONTINUE: u8 = 1;
//// Remount fs read-only
const EXT2_ERRORS_RO: u8 = 2;
//// Panic
const EXT2_ERRORS_PANIC: u8 = 3;
const EXT2_ERRORS_DEFAULT: u8 = EXT2_ERRORS_CONTINUE;
////
//// Codes for operating systems
////
const EXT2_OS_LINUX: u8 = 0;
const EXT2_OS_HURD: u8 = 1;
const EXT2_OBSO_OS_MASIX: u8 = 2;
const EXT2_OS_FREEBSD: u8 = 3;
const EXT2_OS_LITES: u8 = 4;
////
//// Revision levels
////
//// The good old (original) format
const EXT2_GOOD_OLD_REV: u8 = 0;
//// V2 format w/ dynamic inode sizes
const EXT2_DYNAMIC_REV: u8 = 1;
const EXT2_CURRENT_REV: u8 = EXT2_GOOD_OLD_REV;
const EXT2_MAX_SUPP_REV: u8 = EXT2_DYNAMIC_REV;
const EXT2_GOOD_OLD_INODE_SIZE: u8 = 128;
////
//// Journal inode backup types
////
const EXT3_JNL_BACKUP_BLOCKS: u8 = 1;
////
//// Feature set definitions
////
const EXT2_FEATURE_COMPAT_DIR_PREALLOC: u16 = 0x0001;
const EXT2_FEATURE_COMPAT_IMAGIC_INODES: u16 = 0x0002;
const EXT3_FEATURE_COMPAT_HAS_JOURNAL: u16 = 0x0004;
const EXT2_FEATURE_COMPAT_EXT_ATTR: u16 = 0x0008;
const EXT2_FEATURE_COMPAT_RESIZE_INODE: u16 = 0x0010;
const EXT2_FEATURE_COMPAT_DIR_INDEX: u16 = 0x0020;
const EXT2_FEATURE_COMPAT_LAZY_BG: u16 = 0x0040;
const EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER: u16 = 0x0001;
const EXT2_FEATURE_RO_COMPAT_LARGE_FILE: u16 = 0x0002;
const EXT4_FEATURE_RO_COMPAT_HUGE_FILE: u16 = 0x0008;
const EXT4_FEATURE_RO_COMPAT_GDT_CSUM: u16 = 0x0010;
const EXT4_FEATURE_RO_COMPAT_DIR_NLINK: u16 = 0x0020;
const EXT4_FEATURE_RO_COMPAT_EXTRA_ISIZE: u16 = 0x0040;
const EXT2_FEATURE_INCOMPAT_COMPRESSION: u16 = 0x0001;
const EXT2_FEATURE_INCOMPAT_FILETYPE: u16 = 0x0002;
//// Needs recovery
const EXT3_FEATURE_INCOMPAT_RECOVER: u16 = 0x0004;
//// Journal device
const EXT3_FEATURE_INCOMPAT_JOURNAL_DEV: u16 = 0x0008;
const EXT2_FEATURE_INCOMPAT_META_BG: u16 = 0x0010;
const EXT3_FEATURE_INCOMPAT_EXTENTS: u16 = 0x0040;
const EXT4_FEATURE_INCOMPAT_64BIT: u16 = 0x0080;
const EXT4_FEATURE_INCOMPAT_MMP: u16 = 0x0100;
const EXT4_FEATURE_INCOMPAT_FLEX_BG: u16 = 0x0200;
const EXT2_FEATURE_COMPAT_SUPP: u8 = 0;
const EXT2_FEATURE_INCOMPAT_SUPP: u16 = EXT2_FEATURE_INCOMPAT_FILETYPE;
const EXT2_FEATURE_RO_COMPAT_SUPP: u16 =
    EXT2_FEATURE_RO_COMPAT_SPARSE_SUPER | EXT2_FEATURE_RO_COMPAT_LARGE_FILE | EXT4_FEATURE_RO_COMPAT_DIR_NLINK;
////
//// Default values for user and/or group using reserved blocks
////
const EXT2_DEF_RESUID: u8 = 0;
const EXT2_DEF_RESGID: u8 = 0;
////
//// Default mount options
////
const EXT2_DEFM_DEBUG: u16 = 0x0001;
const EXT2_DEFM_BSDGROUPS: u16 = 0x0002;
const EXT2_DEFM_XATTR_USER: u16 = 0x0004;
const EXT2_DEFM_ACL: u16 = 0x0008;
const EXT2_DEFM_UID16: u16 = 0x0010;
const EXT3_DEFM_JMODE: u16 = 0x0060;
const EXT3_DEFM_JMODE_DATA: u16 = 0x0020;
const EXT3_DEFM_JMODE_ORDERED: u16 = 0x0040;
const EXT3_DEFM_JMODE_WBACK: u16 = 0x0060;
const EXT2_NAME_LEN: usize = 255;
////
//// Ext2 directory file types.  Only the low 3 bits are used.  The
//// other bits are reserved for now.
////
const EXT2_FT_UNKNOWN: u8 = 0;
const EXT2_FT_REG_FILE: u8 = 1;
const EXT2_FT_DIR: u8 = 2;
const EXT2_FT_CHRDEV: u8 = 3;
const EXT2_FT_BLKDEV: u8 = 4;
const EXT2_FT_FIFO: u8 = 5;
const EXT2_FT_SOCK: u8 = 6;
const EXT2_FT_SYMLINK: u8 = 7;
const EXT2_FT_MAX: u8 = 8;
////
//// EXT2_DIR_PAD defines the directory entries boundaries
//// NOTE: It must be a multiple of 4
////
const EXT2_DIR_PAD: u8 = 4;
const EXT2_DIR_ROUND: u8 = EXT2_DIR_PAD - 1;
//// ASCII for MMP
const EXT2_MMP_MAGIC: u32 = 0x004D4D50;
//// Value of mmp_seq for clean unmount
const EXT2_MMP_CLEAN: u32 = 0xFF4D4D50;
//// Value of mmp_seq when being fscked
const EXT2_MMP_FSCK_ON: u32 = 0xE24D4D50;
////
//// Interval in number of seconds to update the MMP sequence number.
////
const EXT2_MMP_DEF_INTERVAL: u8 = 5;

//// Header of Access Control Lists
#[repr(C)]
pub struct Ext2AclHeader {
    aclh_size: u32,
    aclh_file_count: u32,
    aclh_acle_count: u32,
    aclh_first_acle: u32,
}

//// Access Control List Entry
#[repr(C)]
pub struct Ext2AclEntry {
    acle_size: u32,
    //// Access permissions
    acle_perms: u16,
    //// Type of entry
    acle_type: u16,
    //// User or group identity
    acle_tag: u16,
    acle_pad1: u16,
    //// Pointer on next entry for the same inode or on next free entry
    acle_next: u32,
}

//// Structure of a blocks group descriptor
#[repr(C)]
pub struct Ext2GroupDesc {
    //// Blocks bitmap block
    bg_block_bitmap: u32,
    //// Inodes bitmap block
    bg_inode_bitmap: u32,
    //// Inodes table block
    bg_inode_table: u32,
    //// Free blocks count
    bg_free_blocks_count: u16,
    //// Free inodes count
    bg_free_inodes_count: u16,
    //// Directories count
    bg_used_dirs_count: u16,
    bg_flags: u16,
    bg_reserved: [u32; 2],
    //// Unused inodes count
    bg_itable_unused: u16,
    //// crc16(s_uuid+grouo_num+group_desc)
    bg_checksum: u16,
}

#[repr(C)]
pub struct Ext4GroupDesc {
    //// Blocks bitmap block
    bg_block_bitmap: u32,
    //// Inodes bitmap block
    bg_inode_bitmap: u32,
    //// Inodes table block
    bg_inode_table: u32,
    //// Free blocks count
    bg_free_blocks_count: u16,
    //// Free inodes count
    bg_free_inodes_count: u16,
    //// Directories count
    bg_used_dirs_count: u16,
    bg_flags: u16,
    bg_reserved: [u32; 2],
    //// Unused inodes count
    bg_itable_unused: u16,
    //// crc16(s_uuid+grouo_num+group_desc)
    bg_checksum: u16,
    //// Blocks bitmap block MSB
    bg_block_bitmap_hi: u32,
    //// Inodes bitmap block MSB
    bg_inode_bitmap_hi: u32,
    //// Inodes table block MSB
    bg_inode_table_hi: u32,
    //// Free blocks count MSB
    bg_free_blocks_count_hi: u16,
    //// Free inodes count MSB
    bg_free_inodes_count_hi: u16,
    //// Directories count MSB
    bg_used_dirs_count_hi: u16,
    bg_pad: u16,
    bg_reserved2: [u32; 3],
}

////
//// Data structures used by the directory indexing feature
////
//// Note: all of the multibyte integer fields are little endian.
////
////
//// Note: dx_root_info is laid out so that if it should somehow get
//// overlaid by a dirent the two low bits of the hash version will be
//// zero.  Therefore, the hash version mod 4 should never be 0.
//// Sincerely, the paranoia department.
////
#[repr(C)]
pub struct Ext2DxRootInfo {
    reserved_zero: u32,
    hash_version: u8,
    info_length: u8,
    indirect_levels: u8,
    unused_flags: u8,
}

#[repr(C)]
pub struct Ext2DxEntry {
    hash: u32,
    block: u32,
}

#[repr(C)]
pub struct Ext2DxCountlimit {
    limit: u16,
    count: u16,
}

#[repr(C)]
pub struct Ext2NewGroupInput {
    //// Group number for this data
    group: u32,
    //// Absolute block number of block bitmap
    block_bitmap: u32,
    //// Absolute block number of inode bitmap
    inode_bitmap: u32,
    //// Absolute block number of inode table start
    inode_table: u32,
    //// Total number of blocks in this group
    blocks_count: u32,
    //// Number of reserved blocks in this group
    reserved_blocks: u16,
    //// Number of reserved GDT blocks in group
    unused: u16,
}

#[repr(C)]
pub struct Ext4NewGroupInput {
    //// Group number for this data
    group: u32,
    //// Absolute block number of block bitmap
    block_bitmap: u64,
    //// Absolute block number of inode bitmap
    inode_bitmap: u64,
    //// Absolute block number of inode table start
    inode_table: u64,
    //// Total number of blocks in this group
    blocks_count: u32,
    //// Number of reserved blocks in this group
    reserved_blocks: u16,
    unused: u16,
}

//// Structure of an inode on the disk
#[repr(C)]
pub struct Ext2Inode {
    //// File mode
    i_mode: u16,
    //// Low 16 bits of Owner Uid
    i_uid: u16,
    //// Size in bytes
    i_size: u32,
    //// Access time
    i_atime: u32,
    //// Inode change time
    i_ctime: u32,
    //// Modification time
    i_mtime: u32,
    //// Deletion Time
    i_dtime: u32,
    //// Low 16 bits of Group Id
    i_gid: u16,
    //// Links count
    i_links_count: u16,
    //// Blocks count
    i_blocks: u32,
    //// File flags
    i_flags: u32,
    //// OS dependent 1
    osd1: OSDep1,
    //// Pointers to blocks
    i_block: [u32; EXT2_N_BLOCKS],
    //// File version (for NFS)
    i_generation: u32,
    //// File ACL
    i_file_acl: u32,
    //// Directory ACL
    i_dir_acl: u32,
    //// Fragment address
    i_faddr: u32,
    //// OS dependent 2
    osd2: OSDep2,
}

//// Permanent part of an large inode on the disk
#[repr(C)]
pub struct Ext2InodeLarge {
    //// File mode
    i_mode: u16,
    //// Low 16 bits of Owner Uid
    i_uid: u16,
    //// Size in bytes
    i_size: u32,
    //// Access time
    i_atime: u32,
    //// Inode Change time
    i_ctime: u32,
    //// Modification time
    i_mtime: u32,
    //// Deletion Time
    i_dtime: u32,
    //// Low 16 bits of Group Id
    i_gid: u16,
    //// Links count
    i_links_count: u16,
    //// Blocks count
    i_blocks: u32,
    //// File flags
    i_flags: u32,
    /// OS dependent 1
    osd1: OSDep1,
    //// Pointers to blocks
    i_block: [u32; EXT2_N_BLOCKS],
    //// File version (for NFS)
    i_generation: u32,
    //// File ACL
    i_file_acl: u32,
    //// Directory ACL
    i_dir_acl: u32,
    //// Fragment address
    i_faddr: u32,
    //// OS dependent 2
    osd2: OSDep2,
    i_extra_isize: u16,
    i_pad1: u16,
    //// extra Change time (nsec << 2 | epoch)
    i_ctime_extra: u32,
    //// extra Modification time (nsec << 2 | epoch)
    i_mtime_extra: u32,
    //// extra Access time (nsec << 2 | epoch)
    i_atime_extra: u32,
    //// File creation time
    i_crtime: u32,
    //// extra File creation time (nsec << 2 | epoch)
    i_crtime_extra: u32,
    //// high 32 bits for 64-bit version
    i_version_hi: u32,
}

//// Structure of the super block
#[repr(C)]
pub struct Ext2SuperBlock {
    //// Inodes count
    s_inodes_count: u32,
    //// Blocks count
    s_blocks_count: u32,
    //// Reserved blocks count
    s_r_blocks_count: u32,
    //// Free blocks count
    s_free_blocks_count: u32,
    //// Free inodes count
    s_free_inodes_count: u32,
    //// First Data Block
    s_first_data_block: u32,
    //// Block size
    s_log_block_size: u32,
    //// Fragment size
    s_log_frag_size: i32,
    //// # Blocks per group
    s_blocks_per_group: u32,
    //// # Fragments per group
    s_frags_per_group: u32,
    //// # Inodes per group
    s_inodes_per_group: u32,
    //// Mount time
    s_mtime: u32,
    //// Write time
    s_wtime: u32,
    //// Mount count
    s_mnt_count: u16,
    //// Maximal mount count
    s_max_mnt_count: i16,
    //// Magic signature
    s_magic: u16,
    //// File system state
    s_state: u16,
    //// Behaviour when detecting errors
    s_errors: u16,
    //// minor revision level
    s_minor_rev_level: u16,
    //// time of last check
    s_lastcheck: u32,
    //// max. time between checks
    s_checkinterval: u32,
    //// OS
    s_creator_os: u32,
    //// Revision level
    s_rev_level: u32,
    //// Default uid for reserved blocks
    s_def_resuid: u16,
    //// Default gid for reserved blocks
    s_def_resgid: u16,
    ////
    //// These fields are for EXT2_DYNAMIC_REV superblocks only.
    /////
    //// Note: the difference between the compatible feature set and
    //// the incompatible feature set is that if there is a bit set
    //// in the incompatible feature set that the kernel doesn't
    //// know about, it should refuse to mount the filesystem.
    /////
    //// e2fsck's requirements are more strict; if it doesn't know
    //// about a feature in either the compatible or incompatible
    //// feature set, it must abort and not try to meddle with
    //// things it doesn't understand...
    /////
    //// First non-reserved inode
    s_first_ino: u32,
    //// size of inode structure
    s_inode_size: u16,
    //// block group # of this superblock
    s_block_group_nr: u16,
    //// compatible feature set
    s_feature_compat: u32,
    //// incompatible feature set
    s_feature_incompat: u32,
    //// readonly-compatible feature set
    s_feature_ro_compat: u32,
    //// 128-bit uuid for volume
    s_uuid: [u8; 16],
    //// volume name
    s_volume_name: [u8; 16],
    //// directory where last mounted
    s_last_mounted: [u8; 64],
    //// For compression
    s_algorithm_usage_bitmap: u32,
    ////
    //// Performance hints.  Directory preallocation should only
    //// happen if the EXT2_FEATURE_COMPAT_DIR_PREALLOC flag is on.
    /////
    //// Nr of blocks to try to preallocate
    s_prealloc_blocks: u8,
    //// Nr to preallocate for dirs
    s_prealloc_dir_blocks: u8,
    //// Per group table for online growth
    s_reserved_gdt_blocks: u16,
    ////
    //// Journaling support valid if EXT2_FEATURE_COMPAT_HAS_JOURNAL set.
    /////
    //// uuid of journal superblock
    s_journal_uuid: [u8; 16],
    //// inode number of journal file
    s_journal_inum: u32,
    //// device number of journal file
    s_journal_dev: u32,
    //// start of list of inodes to delete
    s_last_orphan: u32,
    //// HTREE hash seed
    s_hash_seed: [u32; 4],
    //// Default hash version to use
    s_def_hash_version: u8,
    //// Default type of journal backup
    s_jnl_backup_type: u8,
    //// Group desc. size: INCOMPAT_64BIT
    s_desc_size: u16,
    s_default_mount_opts: u32,
    //// First metablock group
    s_first_meta_bg: u32,
    //// When the filesystem was created
    s_mkfs_time: u32,
    //// Backup of the journal inode
    s_jnl_blocks: [u32; 17],
    //// Blocks count high 32bits
    s_blocks_count_hi: u32,
    //// Reserved blocks count high 32 bits
    s_r_blocks_count_hi: u32,
    //// Free blocks count
    s_free_blocks_hi: u32,
    //// All inodes have at least # bytes
    s_min_extra_isize: u16,
    //// New inodes should reserve # bytes
    s_want_extra_isize: u16,
    //// Miscellaneous flags
    s_flags: u32,
    //// RAID stride
    s_raid_stride: u16,
    //// # seconds to wait in MMP checking
    s_mmp_interval: u16,
    //// Block for multi-mount protection
    s_mmp_block: u64,
    //// blocks on all data disks (N*stride)
    s_raid_stripe_width: u32,
    //// FLEX_BG group size
    s_log_groups_per_flex: u8,
    s_reserved_char_pad: u8,
    //// Padding to next 32bits
    s_reserved_pad: u16,
    //// Padding to the end of the block
    s_reserved: [u32; 162],
}

//// Structure of a directory entry
#[repr(C)]
pub struct Ext2DirEntry {
    //// Inode number
    inode: u32,
    //// Directory entry length
    rec_len: u16,
    //// Name length
    name_len: u16,
    //// Filename
    name: [u8; EXT2_NAME_LEN],
}

//// The new version of the directory entry. Since EXT2 structures are
//// stored in intel byte order, and the name_len field could never be
//// bigger than 255 chars, it's safe to reclaim the extra byte for the
//// file_type field.
#[repr(C)]
pub struct Ext2DirEntry2 {
    //// Inode number
    inode: u32,
    //// Directory entry length
    rec_len: u16,
    //// Name length
    name_len: u8,
    file_type: u8,
    //// Filename
    name: [u8; EXT2_NAME_LEN],
}

//// This structure will be used for multiple mount protection. It will be
//// written into the block number saved in the s_mmp_block field in the
//// superblock.
#[repr(C)]
pub struct MmpStruct {
    mmp_magic: u32,
    mmp_seq: u32,
    mmp_time: u64,
    mmp_nodename: [u8; 64],
    mmp_bdevname: [u8; 32],
    mmp_interval: u16,
    mmp_pad1: u16,
    mmp_pad2: u32,
}

#[repr(C)]
pub union OSDep1 {
    l_i_version: u32,
    h_i_translator: u32,
}

#[repr(C)]
pub union OSDep2 {
    linux2: Linux2,
    hurd2: Hurd2,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Linux2 {
    l_i_blocks_hi: u16,
    l_i_file_acl_high: u16,
    l_i_uid_high: u16,
    l_i_gid_high: u16,
    l_i_reserved2: u32,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Hurd2 {
    /* Fragment number */
    h_i_frag: u8,
    /* Fragment size */
    h_i_fsize: u8,
    h_i_mode_high: u16,
    h_i_uid_high: u16,
    h_i_gid_high: u16,
    h_i_author: u32,
}
