melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  f4 d2 f4 bes2 d4 c4( bes4) a4 bes2 \break

  % Line 2
  d4 c2 f,4 g4( f4) ees4 f2 \break

  % Line 3
  c'4 a2 f4 bes2 d4 d4( c4) bes4 bes4( a4) \break

  % Line 4
  f4 bes2 d4 c4( bes4) a4 bes2 \break

  % Line 5
  d,4 d2 g4 g4( fis4) g4 a4( g4) fis4 g2 \break

  % Line 6
  a4 bes2 a8( g8) f2 ees4 d2 \break

  % Line 7
  f4 f2 bes4 a4( g4) g4 c2 bes4 bes4( a4) \break

  % Line 8
  f4 bes2 d4 c4( bes4) a4 bes2 \bar "|."
}
