melody = \relative c' {
  \clef treble
  \key c \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  a'2 c2 b2 a4 a4 b4 c4 d4 c4 b2 a2 r2 \break

  % Line 2
  d2 c2 d2 a4 c4 b4 a4 a4 gis4 a2 r2 \break

  % Line 3
  a2 a4 g4 f4 f4 bes4 a4 g4 f4 e2 d2 r2 \break

  % Line 4
  f2 f4 g4 a2 c2 b4 a4 b4 c4 d2 r2 \break

  % Line 5
  a2 b4 d4 c4 b4 a4 a4 g4 f4 g2 f2 r2 \break

  % Line 6
  a2 g4 f4 e2 e2 d4 c4 f2 f2 e2 r2 \break

  % Line 7
  a2 g4 a4 d,4 a'4 c4 a4 g4 f4 e2 d1 \bar "|."
}
